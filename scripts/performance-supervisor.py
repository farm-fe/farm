"""Single-command, standard-library OS supervision; never a cross-run budget store."""
# cspell:words ctypes wintypes pswpin pswpout maxrss sysctl pageouts swapouts swapins memorystatus sysconf winapi msvcrt psapi STARTF USESTDHANDLES WNOHANG killpg waitpid pgid rusage osfhandle cmdline lstart ppid pageout swapusage ulonglong STARTUPINFO winerror nonpaged getsid geteuid euid uid
import ctypes
import json
import os
import re
import shutil
import signal
import stat
import subprocess
import sys
import threading
import time


def pressure_snapshot(directory):
    result = {"diskFreeBytes": shutil.disk_usage(directory).free, "availableBytes": None}
    if sys.platform == "darwin":
        level = int(subprocess.check_output(["/usr/sbin/sysctl", "-n", "kern.memorystatus_vm_pressure_level"], text=True, timeout=2).strip())
        if level not in (1, 2, 4):
            raise RuntimeError("Unknown Darwin pressure level")
        result["pressure"] = {"state": {1: "normal", 2: "warning", 4: "critical"}[level], "raw": level, "source": "kern.memorystatus_vm_pressure_level"}
        vm = subprocess.check_output(["/usr/bin/vm_stat"], text=True, timeout=2)
        page_size = re.search(r"page size of (\d+) bytes", vm)
        if not page_size:
            raise RuntimeError("Unknown Darwin page size")
        counters = dict((key, int(value)) for key, value in re.findall(r"^([^:\n]+):\s+(\d+)\.", vm, re.M))
        if "Pageouts" not in counters:
            raise RuntimeError("Missing Darwin pageout counter")
        swap = subprocess.check_output(["/usr/sbin/sysctl", "-n", "vm.swapusage"], text=True, timeout=2)
        used = re.search(r"used\s*=\s*([\d.]+)([KMG])", swap)
        if not used:
            raise RuntimeError("Unknown Darwin swap usage")
        result["swap"] = {"usedBytes": float(used[1]) * (1024 ** {"K": 1, "M": 2, "G": 3}[used[2]]), "pageouts": counters["Pageouts"], "swapins": counters.get("Swapins"), "swapouts": counters.get("Swapouts"), "pageSize": int(page_size[1])}
    elif sys.platform.startswith("linux"):
        with open("/proc/meminfo", encoding="utf-8") as source:
            memory = dict((key, int(value) * 1024) for key, value in re.findall(r"^(\w+):\s+(\d+) kB", source.read(), re.M))
        def read_psi():
            with open("/proc/pressure/memory", encoding="utf-8") as source:
                return dict((kind, {"avg10": float(average), "total": int(total)}) for kind, average, total in re.findall(r"^(some|full) avg10=([\d.]+).*total=(\d+)$", source.read(), re.M))
        previous = read_psi(); time.sleep(0.05); psi = read_psi()
        if set(previous) != {"some", "full"} or set(psi) != {"some", "full"} or "MemAvailable" not in memory:
            raise RuntimeError("Missing Linux memory pressure/available probe")
        deltas = dict((kind, psi[kind]["total"] - previous[kind]["total"]) for kind in ("some", "full"))
        if any(value < 0 for value in deltas.values()):
            raise RuntimeError("Linux PSI counters moved backwards")
        result["pressure"] = {"state": "normal" if not any(deltas.values()) else "warning", "source": "Linux PSI: stop on observed memory stalls", "counters": psi, "stallDeltaMicroseconds": deltas, "observationWindowMs": 50}
        result["availableBytes"] = memory["MemAvailable"]
        with open("/proc/vmstat", encoding="utf-8") as source:
            counters = dict((key, int(value)) for key, value in re.findall(r"^(pswpin|pswpout) (\d+)$", source.read(), re.M))
        if set(counters) != {"pswpin", "pswpout"}:
            raise RuntimeError("Missing Linux paging counters")
        result["swap"] = {"usedBytes": memory["SwapTotal"] - memory["SwapFree"], "swapins": counters.get("pswpin"), "pageouts": counters.get("pswpout"), "pageSize": os.sysconf("SC_PAGE_SIZE")}
    elif os.name == "nt":
        from ctypes import wintypes as w
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)

        class Memory(ctypes.Structure):
            _fields_ = [("length", w.DWORD), ("load", w.DWORD)] + [(name, ctypes.c_ulonglong) for name in ["total", "available", "commitTotal", "commitAvailable", "virtualTotal", "virtualAvailable", "extended"]]

        memory = Memory(); memory.length = ctypes.sizeof(memory)
        if not kernel.GlobalMemoryStatusEx(ctypes.byref(memory)):
            raise ctypes.WinError(ctypes.get_last_error())
        kernel.CreateMemoryResourceNotification.restype = w.HANDLE
        notification = kernel.CreateMemoryResourceNotification(0)
        if not notification:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            low = w.BOOL()
            if not kernel.QueryMemoryResourceNotification(w.HANDLE(notification), ctypes.byref(low)):
                raise ctypes.WinError(ctypes.get_last_error())
        finally:
            kernel.CloseHandle(w.HANDLE(notification))
        result["availableBytes"] = memory.available
        result["pressure"] = {"state": "warning" if low.value else "normal", "source": "Windows LowMemoryResourceNotification", "raw": bool(low.value)}
        result["swap"] = {"usedBytes": None, "pageouts": None, "reason": "No verified cumulative paging counter; commit/pagefile values are not substituted for swap or RSS"}
    else:
        raise RuntimeError("Unsupported pressure platform")
    return result


class ResourceUnavailableError(RuntimeError):
    pass


class ResourceInsufficientError(RuntimeError):
    pass


def verified_snapshot(directory):
    try:
        return pressure_snapshot(directory)
    except Exception as error:
        raise ResourceUnavailableError(str(error)) from error


def admit(snapshot, policy, require_available=True):
    if snapshot["pressure"]["state"] != "normal":
        raise ResourceInsufficientError("Memory pressure is not normal")
    if snapshot["diskFreeBytes"] < policy["diskFloorBytes"]:
        raise ResourceInsufficientError("Disk safety reserve reached")
    minimum = policy.get("requiredAvailableBytes")
    if require_available and minimum and snapshot["availableBytes"] is not None and snapshot["availableBytes"] < minimum:
        raise ResourceInsufficientError("Measured bootstrap memory admission budget not met")


def unix_group_rss(groups):
    rows = subprocess.check_output(["/bin/ps", "-axo", "pid=,pgid=,rss=,time="], text=True, timeout=2)
    values = [(int(rss) * 1024, cpu) for pid, pgid, rss, cpu in re.findall(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+([\d:.-]+)\s*$", rows, re.M) if int(pgid) in groups]
    def seconds(value):
        days, clock = value.split("-", 1) if "-" in value else ("0", value)
        total = 0.0
        for part in clock.split(":"):
            total = total * 60 + float(part)
        return int(days) * 86400 + total
    return {"rssBytes": sum(value[0] for value in values), "liveProcessCpuSeconds": sum(seconds(value[1]) for value in values)}


class WindowsJob:
    """Suspended launch + kill-on-close Job Object; no WMI or unrelated PID kills."""
    def __init__(self, argv, cwd, log):
        import _winapi
        import msvcrt
        from ctypes import wintypes as w
        self.process = None; self.job = None; self.assigned = False
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.psapi = ctypes.WinDLL("psapi", use_last_error=True)
        self.kernel.CreateJobObjectW.restype = w.HANDLE
        self.job = self.kernel.CreateJobObjectW(None, None)
        if not self.job:
            raise ctypes.WinError(ctypes.get_last_error())

        class Basic(ctypes.Structure):
            _fields_ = [("processTime", ctypes.c_longlong), ("jobTime", ctypes.c_longlong), ("flags", w.DWORD), ("minWork", ctypes.c_size_t), ("maxWork", ctypes.c_size_t), ("activeLimit", w.DWORD), ("affinity", ctypes.c_size_t), ("priority", w.DWORD), ("scheduling", w.DWORD)]
        class Io(ctypes.Structure):
            _fields_ = [(name, ctypes.c_ulonglong) for name in ["readOps", "writeOps", "otherOps", "readBytes", "writeBytes", "otherBytes"]]
        class Extended(ctypes.Structure):
            _fields_ = [("basic", Basic), ("io", Io)] + [(name, ctypes.c_size_t) for name in ["processLimit", "jobLimit", "processPeak", "jobPeak"]]
        self.info = Extended(); self.info.basic.flags = 0x2000  # JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        if not self.kernel.SetInformationJobObject(w.HANDLE(self.job), 9, ctypes.byref(self.info), ctypes.sizeof(self.info)):
            raise ctypes.WinError(ctypes.get_last_error())
        startup = subprocess.STARTUPINFO(); startup.dwFlags = subprocess.STARTF_USESTDHANDLES
        self.input = open(os.devnull, "rb")
        for fd in (self.input.fileno(), log.fileno()):
            os.set_inheritable(fd, True)
        startup.hStdInput = msvcrt.get_osfhandle(self.input.fileno())
        startup.hStdOutput = startup.hStdError = msvcrt.get_osfhandle(log.fileno())
        self.process, thread, self.pid, _ = _winapi.CreateProcess(argv[0], subprocess.list2cmdline(argv), None, None, True, 0x4 | 0x200 | 0x08000000, None, cwd, startup)
        try:
            if not self.kernel.AssignProcessToJobObject(w.HANDLE(self.job), w.HANDLE(self.process)):
                self.kernel.TerminateProcess(w.HANDLE(self.process), 1)
                raise ctypes.WinError(ctypes.get_last_error())
            self.assigned = True
            if self.kernel.ResumeThread(w.HANDLE(thread)) == -1:
                raise ctypes.WinError(ctypes.get_last_error())
        finally:
            self.kernel.CloseHandle(w.HANDLE(thread))
        self.kernel.OpenProcess.restype = w.HANDLE

    def rss(self):
        from ctypes import wintypes as w
        buffer = ctypes.create_string_buffer(8 + ctypes.sizeof(ctypes.c_void_p) * 4096)
        if not self.kernel.QueryInformationJobObject(w.HANDLE(self.job), 3, buffer, len(buffer), None):
            raise ctypes.WinError(ctypes.get_last_error())
        count = ctypes.cast(buffer, ctypes.POINTER(w.DWORD))[1]
        if count > 4096:
            raise RuntimeError("Owned process count exceeds bound")
        ids = (ctypes.c_size_t * count).from_buffer(buffer, 8)
        measurements = []
        for pid in ids:
            try:
                measurements.append(self.memory(int(pid)))
            except OSError as error:
                if error.winerror not in (87, 1168):
                    raise
        return {"rssBytes": sum(row[0] for row in measurements), "liveProcessCpuSeconds": sum(row[2] for row in measurements)}

    def memory(self, pid=None):
        from ctypes import wintypes as w
        class Counters(ctypes.Structure):
            _fields_ = [("size", w.DWORD), ("faults", w.DWORD)] + [(name, ctypes.c_size_t) for name in ["peak", "working", "peakPaged", "paged", "peakNonpaged", "nonpaged", "pagefile", "peakPagefile"]]
        handle = self.process if pid is None or pid == self.pid else self.kernel.OpenProcess(0x410, False, pid)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            info = Counters(); info.size = ctypes.sizeof(info)
            if not self.psapi.GetProcessMemoryInfo(w.HANDLE(handle), ctypes.byref(info), info.size):
                raise ctypes.WinError(ctypes.get_last_error())
            created = w.FILETIME(); exited = w.FILETIME(); kernel = w.FILETIME(); user = w.FILETIME()
            if not self.kernel.GetProcessTimes(w.HANDLE(handle), ctypes.byref(created), ctypes.byref(exited), ctypes.byref(kernel), ctypes.byref(user)):
                raise ctypes.WinError(ctypes.get_last_error())
            cpu = ((kernel.dwHighDateTime << 32) + kernel.dwLowDateTime + (user.dwHighDateTime << 32) + user.dwLowDateTime) / 10000000
            return info.working, info.peak, cpu
        finally:
            if handle != self.process:
                self.kernel.CloseHandle(w.HANDLE(handle))

    def poll(self):
        from ctypes import wintypes as w
        code = w.DWORD()
        if not self.kernel.GetExitCodeProcess(w.HANDLE(self.process), ctypes.byref(code)):
            raise ctypes.WinError(ctypes.get_last_error())
        return None if code.value == 259 else code.value

    def cleanup(self):
        from ctypes import wintypes as w
        if self.process and not self.assigned:
            self.kernel.TerminateProcess(w.HANDLE(self.process), 1)
            if self.kernel.WaitForSingleObject(w.HANDLE(self.process), 5000) != 0:
                return False
        if not self.job:
            return self.process is None
        if not self.kernel.TerminateJobObject(w.HANDLE(self.job), 1):
            raise ctypes.WinError(ctypes.get_last_error())
        for _ in range(50):
            buffer = ctypes.create_string_buffer(48)
            if not self.kernel.QueryInformationJobObject(w.HANDLE(self.job), 1, buffer, len(buffer), None):
                raise ctypes.WinError(ctypes.get_last_error())
            if ctypes.c_uint32.from_buffer(buffer, 40).value == 0:  # ActiveProcesses
                return True
            time.sleep(0.05)
        return False


def run(config):
    result = {"status": "invalid", "cleanupConfirmed": False, "samples": 0, "peakObservedGroupRssBytes": 0, "kernelPeakRssBytes": None}
    child = None; job = None
    started = None; usage = None
    completed = threading.Event(); completion = {}; waiter_started = False
    cleanup_end = None
    browser_group = None
    def probe_budget(seconds):
        if cleanup_end is None:
            return seconds
        remaining = cleanup_end - time.monotonic()
        if remaining <= 0:
            raise RuntimeError("Owned cleanup deadline exhausted")
        return min(seconds, remaining)
    def browser_owner():
        if not config.get("browserOwnerFile") or not child:
            return None
        info = os.lstat(config["browserOwnerFile"])
        if not stat.S_ISREG(info.st_mode) or info.st_size > 4096:
            raise RuntimeError("Invalid owned browser receipt")
        with open(config["browserOwnerFile"], encoding="utf-8") as source:
            owner = json.load(source)
        if owner.get("rootPid") != child.pid:
            if owner.get("state") == "not-started":
                return owner
            raise RuntimeError("Browser receipt root mismatch")
        return owner
    def register_browser():
        nonlocal browser_group
        if browser_group:
            return
        owner = browser_owner()
        if not owner or owner.get("state") != "open":
            return
        pid = owner.get("browserPid")
        if not isinstance(pid, int) or pid <= 1:
            raise RuntimeError("Invalid owned browser PID")
        row = subprocess.check_output(["/bin/ps", "-p", str(pid), "-o", "ppid=,pgid=,lstart="], text=True, timeout=probe_budget(2)).strip().split(None, 2)
        if len(row) != 3 or int(row[0]) != child.pid or int(row[1]) != pid:
            raise RuntimeError("Browser group is not a direct owned child")
        browser_group = {"pid": pid, "started": row[2], "parentVerified": True}
        result["browserGroup"] = browser_group
    def group_state(group):
        # State/identity queries do not send signals. Keep only owned-group
        # records in the result; do not log unrelated processes or commands.
        output = subprocess.check_output(["/bin/ps", "-axo", "pid=,ppid=,pgid=,uid=,stat=,lstart="], text=True, timeout=probe_budget(1))
        rows = {}
        for pid, ppid, pgid, uid, state, creation in re.findall(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+(.+?)\s*$", output, re.M):
            rows[int(pid)] = {"pid": int(pid), "ppid": int(ppid), "pgid": int(pgid), "uid": int(uid), "state": state, "created": creation}
        return rows, [row for row in rows.values() if row["pgid"] == group]
    def clear_group(group, scope):
        proof = {"operation": "ps-group-state", "scope": scope, "pgid": group, "rootPid": child.pid, "euid": os.geteuid(), "cleanupConfirmed": False}
        result.setdefault("cleanupScopes", []).append(proof)
        rows, members = group_state(group)
        proof["before"] = members[:128]
        if scope == "root":
            identity = result.get("rootOwnership")
            if not identity or not identity.get("verified"):
                raise RuntimeError("Root creation ownership is unconfirmed; refusing group signal")
            leader = rows.get(group)
            if leader and leader["created"] != identity["created"]:
                raise RuntimeError("Root PID creation identity changed; refusing group signal")
        proof["operation"] = "validate-group-ownership"
        if len(members) > 4096:
            raise RuntimeError("Owned group exceeds observation bound")
        for member in members:
            if member["uid"] != os.geteuid():
                raise RuntimeError("Group ownership UID mismatch; refusing signal")
            try:
                proof["operation"] = "getsid"; proof["targetPid"] = member["pid"]
                session = os.getsid(member["pid"])
                if session != group:
                    raise RuntimeError("Group is outside the spawned owned session; refusing signal")
            except ProcessLookupError:
                pass  # A disappearing member is checked again in the final table.
        proof["ownershipVerified"] = True
        known = {row["pid"] for row in members}
        if scope == "root":
            known.add(child.pid)
        if members and any(not row["state"].startswith("Z") for row in members):
            proof["operation"] = "killpg-SIGKILL"
            try:
                os.killpg(group, signal.SIGKILL)  # At most one terminating signal.
                proof["signalResult"] = "sent"
            except ProcessLookupError:
                proof["signalResult"] = "already-absent"
            except PermissionError as error:
                proof["signalResult"] = "denied"; proof["errno"] = error.errno; proof["error"] = str(error)
                proof["deniedOperation"] = "killpg-SIGKILL"; proof["operation"] = "ps-after-denied-SIGKILL"
                _, current = group_state(group)
                proof["afterDeniedSignal"] = current[:128]
                if any(not row["state"].startswith("Z") for row in current):
                    result["cleanupError"] = f"killpg-SIGKILL denied for owned PGID {group}: {error}"
                    result["cleanupErrno"] = error.errno
                    return False  # No retry or alternate termination API on EPERM.
                # A zombie-only/empty group has no signalable members. Reap/wait
                # and prove disappearance; EPERM itself is never success.
        end = min(time.monotonic() + 3, cleanup_end)
        while time.monotonic() < end:
            proof["operation"] = "ps-state-and-reap"
            if scope == "root" and not waiter_started and not completed.is_set():
                # Only a thread-start failure can use this fallback. Never race
                # a second waitpid against the active wait4 reaper thread.
                proof["operation"] = "wait4-WNOHANG"
                pid, status, _ = os.wait4(child.pid, os.WNOHANG)
                if pid:
                    child.returncode = os.waitstatus_to_exitcode(status)
                    completion["code"] = child.returncode; completed.set()
            proof["operation"] = "ps-state-and-reap"
            rows, current = group_state(group)
            remaining = [rows[pid] for pid in known if pid in rows]
            proof["remaining"] = remaining[:128]; proof["groupMembers"] = current[:128]
            proof["rootReaped"] = scope != "root" or (completed.is_set() and "code" in completion)
            if not current and not remaining and proof["rootReaped"]:
                proof["cleanupConfirmed"] = True
                return True
            completed.wait(0.05) if not completed.is_set() else time.sleep(0.05)
        proof["error"] = "Owned group/PIDs did not disappear within bounded reap wait"
        return False
    def cleanup():
        nonlocal cleanup_end
        cleanup_end = time.monotonic() + 6
        if job:
            return job.cleanup()
        if not child:
            return True
        # Playwright intentionally creates a separate Unix browser process group.
        # A receipt alone is not ownership: register against its live parent and
        # creation identity before killing the build's group.
        browser_clean = True
        try:
            register_browser()
            if config.get("browserOwnerFile"):
                if browser_group:
                    pid = browser_group["pid"]
                    row = subprocess.run(["/bin/ps", "-p", str(pid), "-o", "lstart="], text=True, capture_output=True, timeout=probe_budget(2))
                    if row.stderr.strip() or (row.returncode == 0 and row.stdout.strip() != browser_group["started"]):
                        raise RuntimeError("Browser PID creation identity unavailable/changed; refusing kill")
                    browser_clean = clear_group(pid, "browser")
                else:
                    # Public BrowserServer.close must finish if the short-lived
                    # browser exited before registration. Failed launches remain
                    # unconfirmed. Never kill an unverified group from JSON alone.
                    owner = browser_owner()
                    browser_clean = False
                    if owner is not None and owner.get("state") == "closed" and isinstance(owner.get("browserPid"), int) and owner["browserPid"] > 1:
                        end = min(time.monotonic() + 3, cleanup_end)
                        while time.monotonic() < end:
                            rows, members = group_state(owner["browserPid"])
                            if owner["browserPid"] not in rows and not members:
                                browser_clean = True
                                break
                            time.sleep(0.05)
        except Exception as error:
            browser_clean = False
            result["browserCleanupError"] = str(error)
            result["browserCleanupErrno"] = getattr(error, "errno", None)
        # Even a browser ownership/probe error must stop the proven owned root.
        return clear_group(child.pid, "root") and browser_clean
    def interrupted(*_):
        raise RuntimeError("Supervisor interrupted")
    signal.signal(signal.SIGTERM, interrupted)
    try:
        result["before"] = verified_snapshot(config["cwd"]); admit(result["before"], config["policy"])
        with open(config["log"], "ab", buffering=0) as log:
            started = time.monotonic()
            if os.name == "nt":
                job = WindowsJob.__new__(WindowsJob)
                job.__init__(config["argv"], config["cwd"], log)
            else:
                child = subprocess.Popen(config["argv"], cwd=config["cwd"], stdout=log, stderr=log, start_new_session=True)
            result["rootPid"] = job.pid if job else child.pid
            def wait_for_root():
                try:
                    if job:
                        from ctypes import wintypes as w
                        if job.kernel.WaitForSingleObject(w.HANDLE(job.process), 0xFFFFFFFF) != 0:
                            raise ctypes.WinError(ctypes.get_last_error())
                        completion["code"] = job.poll()
                    else:
                        _, status, root_usage = os.wait4(child.pid, 0)
                        completion["usage"] = root_usage
                        completion["code"] = os.waitstatus_to_exitcode(status)
                        child.returncode = completion["code"]
                    completion["ended"] = time.monotonic()
                except Exception as error:
                    completion["error"] = str(error)
                finally:
                    completed.set()
            threading.Thread(target=wait_for_root, daemon=True).start()
            waiter_started = True
            if child:
                rows, _ = group_state(child.pid)
                identity = rows.get(child.pid)
                if identity:
                    if identity["ppid"] != os.getpid() or identity["pgid"] != child.pid or identity["uid"] != os.geteuid():
                        raise RuntimeError("Spawned root ownership could not be verified")
                    result["rootOwnership"] = {**identity, "verified": True, "observerPid": os.getpid(), "newSessionRequested": True}
                elif completed.is_set() and "code" in completion:
                    result["rootOwnership"] = {"pid": child.pid, "created": None, "verified": True, "leaderAlreadyReaped": True, "observerPid": os.getpid(), "newSessionRequested": True}
                else:
                    raise RuntimeError("Root observation absent without completed reaping")
            previous_pressure = result["before"]["pressure"]
            while True:
                observation = verified_snapshot(config["cwd"])
                if sys.platform.startswith("linux"):
                    for kind in ("some", "full"):
                        if observation["pressure"]["counters"][kind]["total"] != previous_pressure["counters"][kind]["total"]:
                            observation["pressure"]["state"] = "warning"
                    previous_pressure = observation["pressure"]
                result["lastObservation"] = observation
                admit(observation, config["policy"], require_available=False)
                if not job:
                    register_browser()
                observed = job.rss() if job else unix_group_rss({child.pid, *([browser_group["pid"]] if browser_group else [])})
                result["samples"] += 1; result["peakObservedGroupRssBytes"] = max(result["peakObservedGroupRssBytes"], observed["rssBytes"])
                print(json.dumps({"event": "sample", "elapsedMs": (time.monotonic() - started) * 1000, **observed, **observation}), flush=True)
                if completed.is_set():
                    if "error" in completion:
                        raise RuntimeError(completion["error"])
                    result["exitCode"] = completion["code"]
                    usage = completion.get("usage")
                    break
                if (time.monotonic() - started) * 1000 >= config["timeoutMs"]:
                    raise RuntimeError("Supervised command deadline reached")
                completed.wait(1)
            result["durationMs"] = (completion["ended"] - started) * 1000
            if result["durationMs"] > config["timeoutMs"]:
                raise RuntimeError("Child completed after its allocated timeout")
            result["kernelPeakRssBytes"] = job.memory()[1] if job else usage.ru_maxrss * (1 if sys.platform == "darwin" else 1024)
            result["rssSource"] = "Windows PeakWorkingSetSize" if job else "wait4 rusage maxrss (Darwin bytes; Linux KiB converted)"
            result["after"] = verified_snapshot(config["cwd"]); admit(result["after"], config["policy"], require_available=False)
            if result["exitCode"] != 0:
                raise RuntimeError("Supervised child failed")
            if not result["kernelPeakRssBytes"]:
                raise RuntimeError("Kernel peak RSS unavailable")
            result["status"] = "complete"
    except Exception as error:
        result["error"] = str(error)
        result["errorType"] = type(error).__name__
        result["failureKind"] = "resource-unavailable" if isinstance(error, ResourceUnavailableError) else "resource-insufficient" if isinstance(error, ResourceInsufficientError) else "sampling-failure"
    finally:
        # Defer only repeated stop signals while this bounded cleanup completes.
        # The live build has already entered its failure/exit path.
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        if started is not None:
            result["elapsedMs"] = (time.monotonic() - started) * 1000
        result["phase"] = config.get("phase"); result["cwd"] = config["cwd"]; result["timeoutMs"] = config["timeoutMs"]
        try:
            result["cleanupConfirmed"] = cleanup()
        except Exception as error:
            result["cleanupError"] = str(error)
            result["cleanupErrno"] = getattr(error, "errno", None)
            result["cleanupExceptionType"] = type(error).__name__
        if not result["cleanupConfirmed"]:
            result["status"] = "invalid"
        print(json.dumps({"event": "result", **result}), flush=True)
    return result


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--probe":
        print(json.dumps(pressure_snapshot(sys.argv[2])))
    else:
        result = run(json.loads(sys.stdin.readline()))
        sys.exit(0 if result["status"] == "complete" else 1)

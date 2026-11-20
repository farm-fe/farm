// Diagnostic only, Windows x64. Creates and observes only its own child.
// Event layout and continuation semantics:
// https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-debug_event
// https://learn.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-continuedebugevent
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

public static class FarmNativeCrashDebugger {
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
  struct StartupInfo {
    public int cb; public string reserved, desktop, title;
    public uint x,y,xSize,ySize,xChars,yChars,fill,flags;
    public ushort showWindow,reservedSize; public IntPtr reservedBytes,input,output,error;
  }
  [StructLayout(LayoutKind.Sequential)]
  struct ProcessInfo { public IntPtr process,thread; public uint pid,tid; }
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
  static extern bool CreateProcessW(string application, StringBuilder command, IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags, IntPtr environment, string cwd, ref StartupInfo startup, out ProcessInfo info);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool WaitForDebugEvent(IntPtr debugEvent, uint milliseconds);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool ContinueDebugEvent(uint pid, uint tid, uint status);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool TerminateProcess(IntPtr process, uint exitCode);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool SetHandleInformation(IntPtr handle, uint mask, uint flags);

  public class Crash {
    public int ThreadId;
    public bool FirstChance;
    public string Code, Address, Module, ModuleOffset, AccessKind, InvalidAddress, StackError;
    public List<FarmNativeStacks.ThreadTrace> Threads;
  }
  public class Result {
    public uint ProcessId, ExitCode;
    public bool Ready, TimedOut, TerminatedByDiagnostic;
    public string Started, Ended, Executable, Command, Error;
    public List<Crash> Exceptions = new List<Crash>();
  }

  static void Save(Result result, string path) {
    File.WriteAllText(path, JsonSerializer.Serialize(result, new JsonSerializerOptions { IncludeFields=true, WriteIndented=true }));
  }

  public static Result Run(string executable, string arguments, string cwd, string readyUrl, int timeoutSeconds, string symbolPath, string reportPath) {
    if(!Environment.Is64BitProcess) throw new Exception("Only the Windows x64 event layout is supported");
    var result = new Result { Started=DateTime.UtcNow.ToString("o"), Executable=executable, Command=arguments };
    var info = new ProcessInfo();
    var buffer = Marshal.AllocHGlobal(192);
    bool created=false, exited=false;
    using(var output = new FileStream(reportPath+".stdout.log", FileMode.Create, FileAccess.Write, FileShare.ReadWrite))
    using(var input = new FileStream("NUL", FileMode.Open, FileAccess.Read, FileShare.ReadWrite))
    using(var client = new HttpClient { Timeout=TimeSpan.FromMilliseconds(500) }) {
      try {
        IntPtr outputHandle=output.SafeFileHandle.DangerousGetHandle(), inputHandle=input.SafeFileHandle.DangerousGetHandle();
        if(!SetHandleInformation(outputHandle,1,1) || !SetHandleInformation(inputHandle,1,1)) throw new Exception("SetHandleInformation failed: "+Marshal.GetLastWin32Error());
        var startup = new StartupInfo { cb=Marshal.SizeOf<StartupInfo>(), flags=0x100, input=inputHandle, output=outputHandle, error=outputHandle };
        var command = new StringBuilder("\""+executable+"\" "+arguments);
        if(!CreateProcessW(executable,command,IntPtr.Zero,IntPtr.Zero,true,0x08000002,IntPtr.Zero,cwd,ref startup,out info)) throw new Exception("CreateProcess failed: "+Marshal.GetLastWin32Error());
        created=true; result.ProcessId=info.pid; Save(result,reportPath);
        var deadline=DateTime.UtcNow.AddSeconds(timeoutSeconds);
        while(!exited) {
          if(!WaitForDebugEvent(buffer,200)) {
            int error=Marshal.GetLastWin32Error();
            if(error!=121) throw new Exception("WaitForDebugEvent failed: "+error);
            if(!result.TerminatedByDiagnostic) {
              try { using(var response=client.GetAsync(readyUrl).GetAwaiter().GetResult()) result.Ready=response.IsSuccessStatusCode; }
              catch(HttpRequestException) {} catch(System.Threading.Tasks.TaskCanceledException) {}
              if(result.Ready || DateTime.UtcNow>=deadline) {
                result.TimedOut=!result.Ready; result.TerminatedByDiagnostic=true;
                if(!TerminateProcess(info.process,0)) throw new Exception("TerminateProcess failed: "+Marshal.GetLastWin32Error());
              }
            }
            continue;
          }
          uint kind=(uint)Marshal.ReadInt32(buffer,0), pid=(uint)Marshal.ReadInt32(buffer,4), tid=(uint)Marshal.ReadInt32(buffer,8);
          // The union is aligned to eight bytes at offset 16 on Windows x64.
          IntPtr data=IntPtr.Add(buffer,16);
          uint continuation=0x10002;
          try {
            if(kind==1) {
              uint code=(uint)Marshal.ReadInt32(data,0);
              bool firstChance=Marshal.ReadInt32(data,152)!=0;
              if(code!=0x80000003) continuation=0x80010001;
              if(code==0xc0000005 || (!firstChance && code!=0x80000003)) {
                ulong address=(ulong)Marshal.ReadInt64(data,16);
                var crash=new Crash { ThreadId=(int)tid, FirstChance=firstChance, Code="0x"+code.ToString("x"), Address="0x"+address.ToString("x") };
                if(code==0xc0000005 && Marshal.ReadInt32(data,24)>=2) {
                  crash.AccessKind="0x"+((ulong)Marshal.ReadInt64(data,32)).ToString("x");
                  crash.InvalidAddress="0x"+((ulong)Marshal.ReadInt64(data,40)).ToString("x");
                }
                try {
                  foreach(ProcessModule module in Process.GetProcessById((int)pid).Modules) {
                    ulong start=(ulong)module.BaseAddress.ToInt64();
                    if(address>=start && address<start+(ulong)module.ModuleMemorySize) { crash.Module=module.FileName; crash.ModuleOffset="0x"+(address-start).ToString("x"); break; }
                  }
                  crash.Threads=FarmNativeStacks.Capture((int)pid,symbolPath);
                } catch(Exception error) { crash.StackError=error.ToString(); }
                result.Exceptions.Add(crash); Save(result,reportPath);
              }
            } else if(kind==3 || kind==6) {
              // Debugger owns the event's image-file handle. Process/thread
              // event handles are released by ContinueDebugEvent on exit.
              IntPtr file=Marshal.ReadIntPtr(data,0); if(file!=IntPtr.Zero) CloseHandle(file);
            } else if(kind==5) {
              result.ExitCode=(uint)Marshal.ReadInt32(data,0); exited=true;
            }
          } finally {
            if(!ContinueDebugEvent(pid,tid,continuation)) throw new Exception("ContinueDebugEvent failed: "+Marshal.GetLastWin32Error());
          }
        }
      } catch(Exception error) { result.Error=error.ToString(); }
      finally {
        if(created && !exited) { TerminateProcess(info.process,1); result.TerminatedByDiagnostic=true; }
        if(info.process!=IntPtr.Zero) CloseHandle(info.process);
        if(info.thread!=IntPtr.Zero) CloseHandle(info.thread);
        Marshal.FreeHGlobal(buffer);
        result.Ended=DateTime.UtcNow.ToString("o"); Save(result,reportPath);
      }
    }
    return result;
  }
}

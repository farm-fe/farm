import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('./overlay.js', () => ({
  ErrorOverlay: class {},
  overlayId: 'overlay'
}));

class Socket extends EventTarget {
  static CLOSING = 2;
  static CLOSED = 3;
  readyState = 1;
  close() {
    this.readyState = Socket.CLOSED;
    this.dispatchEvent(new Event('close'));
  }
}

describe('HMR connection recovery', () => {
  let windowEvents: EventTarget;
  let reload: ReturnType<typeof vi.fn>;
  let fetchMock: ReturnType<typeof vi.fn>;
  let client: import('./hmr-client.js').HmrClient;

  beforeEach(async () => {
    vi.resetModules();
    vi.useFakeTimers();
    windowEvents = new EventTarget();
    reload = vi.fn();
    fetchMock = vi.fn().mockResolvedValue({});
    const location = {
      port: '9000',
      hostname: 'localhost',
      protocol: 'http:',
      reload
    };
    vi.stubGlobal('window', Object.assign(windowEvents, { location }));
    vi.stubGlobal('location', location);
    vi.stubGlobal(
      'document',
      Object.assign(new EventTarget(), { visibilityState: 'visible' })
    );
    vi.stubGlobal('WebSocket', Socket);
    vi.stubGlobal('fetch', fetchMock);
    vi.stubGlobal('FARM_HMR_HOST', true);
    vi.stubGlobal('FARM_HMR_PROTOCOL', '');
    vi.stubGlobal('FARM_HMR_PATH', '/__hmr');
    const { HmrClient } = await import('./hmr-client.js');
    client = new HmrClient({} as never);
    client.connect();
  });

  afterEach(() => {
    windowEvents.dispatchEvent(new Event('pagehide'));
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('does not reload when navigation closes the socket', async () => {
    windowEvents.dispatchEvent(new Event('pagehide'));
    client.socket.dispatchEvent(new Event('close'));
    await vi.runAllTimersAsync();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(reload).not.toHaveBeenCalled();
  });

  it('does not reload when Firefox closes the socket before pagehide', async () => {
    windowEvents.dispatchEvent(new Event('beforeunload'));
    Object.assign(client.socket, { readyState: Socket.CLOSING });
    await vi.advanceTimersByTimeAsync(0);
    client.socket.dispatchEvent(new Event('close'));
    await vi.runAllTimersAsync();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(reload).not.toHaveBeenCalled();
  });

  it('cancels a pending recovery when navigation starts', async () => {
    let resolvePing: (value: object) => void;
    fetchMock.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolvePing = resolve;
        })
    );
    client.socket.dispatchEvent(new Event('close'));
    expect(fetchMock).toHaveBeenCalledOnce();
    const signal = fetchMock.mock.calls[0][1].signal;
    windowEvents.dispatchEvent(new Event('pagehide'));
    resolvePing({});
    await vi.runAllTimersAsync();
    expect(signal).toBeDefined();
    expect(signal?.aborted).toBe(true);
    expect(reload).not.toHaveBeenCalled();
  });

  it('cancels retry timers during navigation', async () => {
    fetchMock.mockRejectedValue(new Error('offline'));
    client.socket.dispatchEvent(new Event('close'));
    await vi.advanceTimersByTimeAsync(0);
    windowEvents.dispatchEvent(new Event('pagehide'));
    await vi.advanceTimersByTimeAsync(5000);
    expect(fetchMock).toHaveBeenCalledOnce();
    expect(reload).not.toHaveBeenCalled();
  });

  it('reconnects a restored page and ignores its old socket', async () => {
    const oldSocket = client.socket;
    windowEvents.dispatchEvent(new Event('pagehide'));
    const restored = Object.assign(new Event('pageshow'), { persisted: true });
    windowEvents.dispatchEvent(restored);
    expect(client.socket).not.toBe(oldSocket);
    oldSocket.dispatchEvent(new Event('close'));
    await vi.runAllTimersAsync();
    expect(fetchMock).not.toHaveBeenCalled();
    client.socket.dispatchEvent(new Event('close'));
    await vi.runAllTimersAsync();
    expect(reload).toHaveBeenCalledOnce();
  });

  it('removes a pending visibility listener during navigation', async () => {
    fetchMock.mockRejectedValue(new Error('offline'));
    Object.assign(document, { visibilityState: 'hidden' });
    const add = vi.spyOn(document, 'addEventListener');
    const remove = vi.spyOn(document, 'removeEventListener');
    client.socket.dispatchEvent(new Event('close'));
    await vi.advanceTimersByTimeAsync(1000);
    expect(add).toHaveBeenCalledWith('visibilitychange', expect.any(Function));
    windowEvents.dispatchEvent(new Event('pagehide'));
    await vi.advanceTimersByTimeAsync(0);
    expect(remove).toHaveBeenCalledWith(
      'visibilitychange',
      expect.any(Function)
    );
    Object.assign(document, { visibilityState: 'visible' });
    document.dispatchEvent(new Event('visibilitychange'));
    await vi.advanceTimersByTimeAsync(5000);
    expect(fetchMock).toHaveBeenCalledOnce();
    expect(reload).not.toHaveBeenCalled();
  });

  it('still recovers after a canceled beforeunload navigation', async () => {
    const navigation = new Event('beforeunload', { cancelable: true });
    navigation.preventDefault();
    windowEvents.dispatchEvent(navigation);
    await vi.advanceTimersByTimeAsync(0);
    client.socket.dispatchEvent(new Event('close'));
    await vi.runAllTimersAsync();
    expect(reload).toHaveBeenCalledOnce();
  });

  it('reconnects when navigation is canceled during pending server recovery', async () => {
    fetchMock.mockRejectedValue(new Error('offline'));
    const oldSocket = client.socket;
    oldSocket.close();
    await vi.advanceTimersByTimeAsync(0);
    windowEvents.dispatchEvent(new Event('beforeunload', { cancelable: true }));
    await vi.advanceTimersByTimeAsync(0);
    expect(client.socket).not.toBe(oldSocket);
    expect(reload).not.toHaveBeenCalled();
    const interaction = new Event('pointerdown');
    Object.defineProperty(interaction, 'isTrusted', { value: true });
    windowEvents.dispatchEvent(interaction);
    fetchMock.mockResolvedValue({});
    client.socket.close();
    await vi.runAllTimersAsync();
    expect(reload).toHaveBeenCalledOnce();
  });

  it('does not reload if a speculative reconnect closes before pagehide', async () => {
    fetchMock.mockRejectedValue(new Error('offline'));
    client.socket.close();
    await vi.advanceTimersByTimeAsync(0);
    windowEvents.dispatchEvent(new Event('beforeunload'));
    await vi.advanceTimersByTimeAsync(0);
    fetchMock.mockResolvedValue({});
    client.socket.close();
    await vi.advanceTimersByTimeAsync(5000);
    expect(reload).not.toHaveBeenCalled();
    const speculativeSocket = client.socket;
    const interaction = new Event('keydown');
    Object.defineProperty(interaction, 'isTrusted', { value: true });
    windowEvents.dispatchEvent(interaction);
    expect(client.socket).not.toBe(speculativeSocket);
    client.socket.close();
    await vi.runAllTimersAsync();
    expect(reload).toHaveBeenCalledOnce();
  });

  it('recovers from a server close while the page is active', async () => {
    client.closeConnectionGracefully();
    await vi.runAllTimersAsync();
    expect(reload).toHaveBeenCalledOnce();
  });
});

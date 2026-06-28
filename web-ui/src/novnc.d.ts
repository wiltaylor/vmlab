declare module "@novnc/novnc" {
  export default class RFB extends EventTarget {
    constructor(target: HTMLElement, url: string, options?: Record<string, unknown>);
    scaleViewport: boolean;
    clipViewport: boolean;
    disconnect(): void;
    sendCtrlAltDel(): void;
    focus(): void;
  }
}

import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

export async function openTrafficLogWindow() {
  try {
    const existing = await WebviewWindow.getByLabel('traffic-log-window');
    if (existing) {
      await existing.setFocus();
      return;
    }

    const webview = new WebviewWindow('traffic-log-window', {
      url: '/#/traffic-log',
      title: 'Modbus Communication Traffic Log - Modlab',
      width: 850,
      height: 550,
      resizable: true,
      alwaysOnTop: false,
    });

    webview.once('tauri://created', function () {
      console.log('Traffic Log Window created successfully');
    });

    webview.once('tauri://error', function (e) {
      console.error('Error creating Traffic Log Window:', e);
    });
  } catch (err) {
    console.error('Multi-window operation failed:', err);
  }
}

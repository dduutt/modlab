import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export interface UpdateStatus {
  checking: boolean;
  available: boolean;
  version?: string;
  body?: string;
  date?: string;
  error?: string;
  downloading: boolean;
  downloaded: number;
  total: number;
  readyToRelaunch: boolean;
}

export class UpdateService {
  private static activeUpdate: Update | null = null;

  /**
   * 检查是否有新版本
   */
  static async checkForUpdates(): Promise<{ available: boolean; update: Update | null; version?: string; body?: string; date?: string }> {
    try {
      const update = await check();
      this.activeUpdate = update;
      if (update?.available) {
        return {
          available: true,
          update,
          version: update.version,
          body: update.body,
          date: update.date,
        };
      }
      return { available: false, update: null };
    } catch (err: any) {
      console.error('Check update failed:', err);
      throw err;
    }
  }

  /**
   * 下载并安装更新
   */
  static async installUpdate(onProgress?: (downloaded: number, total: number) => void): Promise<void> {
    if (!this.activeUpdate) {
      const res = await check();
      this.activeUpdate = res;
    }

    if (!this.activeUpdate?.available) {
      throw new Error('No update available');
    }

    let downloadedBytes = 0;
    let totalBytes = 0;

    await this.activeUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        totalBytes = event.data.contentLength || 0;
      } else if (event.event === 'Progress') {
        downloadedBytes += event.data.chunkLength;
        if (onProgress) {
          onProgress(downloadedBytes, totalBytes);
        }
      } else if (event.event === 'Finished') {
        if (onProgress && totalBytes > 0) {
          onProgress(totalBytes, totalBytes);
        }
      }
    });
  }

  /**
   * 重启应用以应用更新
   */
  static async restartApp(): Promise<void> {
    await relaunch();
  }
}

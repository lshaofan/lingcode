import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from '../../../stores';
import { Toggle } from '../../../components';
import { useToast } from '../../../components';

export const SystemSettings: React.FC = () => {
  const { settings, updateSetting } = useSettingsStore();
  const toast = useToast();
  const [loading, setLoading] = useState(false);

  // 加载开机自启状态
  useEffect(() => {
    loadAutoLaunchStatus();
  }, []);

  const loadAutoLaunchStatus = async () => {
    try {
      const status = await invoke<boolean>('get_auto_launch');
      if (status !== settings.autoStart) {
        await updateSetting('autoStart', status);
      }
    } catch (error) {
      console.error('Failed to load auto launch status:', error);
    }
  };

  const handleAutoStartChange = async (enabled: boolean) => {
    setLoading(true);
    try {
      await invoke('set_auto_launch', { enable: enabled });
      await updateSetting('autoStart', enabled);
      toast.success(enabled ? '已启用开机自启动' : '已禁用开机自启动');
    } catch (error) {
      toast.error(`设置失败: ${error}`);
      console.error('Failed to set auto launch:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleShowInDockChange = async (enabled: boolean) => {
    setLoading(true);
    try {
      await updateSetting('showInDock', enabled);
      toast.success(enabled ? '应用将在 Dock 中显示' : '应用将仅在托盘显示');
    } catch (error) {
      toast.error(`设置失败: ${error}`);
      console.error('Failed to set show in dock:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-semibold text-gray-900">系统设置</h3>

      <div>
        <h4 className="text-sm font-medium text-gray-500 mb-3">App settings</h4>

        <div className="space-y-4">
          {/* 开机自动启动 */}
          <div className="p-4 bg-gray-50 rounded-lg flex items-center justify-between">
            <div className="flex-1">
              <div className="font-medium text-gray-900">开机自动启动</div>
              <div className="text-sm text-gray-500 mt-1">Launch app at login</div>
            </div>
            <Toggle
              checked={settings.autoStart}
              onChange={handleAutoStartChange}
              disabled={loading}
            />
          </div>

          {/* 在 Dock 中显示 */}
          <div className="p-4 bg-gray-50 rounded-lg flex items-center justify-between">
            <div className="flex-1">
              <div className="font-medium text-gray-900">在 Dock 中显示</div>
              <div className="text-sm text-gray-500 mt-1">Show app in dock</div>
            </div>
            <Toggle
              checked={settings.showInDock}
              onChange={handleShowInDockChange}
              disabled={loading}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

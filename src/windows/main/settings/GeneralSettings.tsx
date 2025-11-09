import React, { useMemo } from 'react';
import { useSettingsStore, useUIStore } from '../../../stores';
import { Button } from '../../../components';
import { getShortcutDisplayParts } from '../../../utils/shortcutFormatter';

export const GeneralSettings: React.FC = () => {
  const { settings } = useSettingsStore();
  const { openLanguageSelector } = useUIStore();

  // 获取快捷键显示部分
  const shortcutParts = useMemo(() => {
    return getShortcutDisplayParts(settings.shortcut || 'Cmd+Shift+S');
  }, [settings.shortcut]);

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-semibold text-gray-900">通用设置</h3>

      {/* 键盘快捷键 */}
      <div className="p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <h4 className="font-medium text-gray-900">键盘快捷键</h4>
              <button className="text-xs text-green-600 hover:text-green-700">
                了解更多 →
              </button>
            </div>
            <p className="text-sm text-gray-600 flex items-center gap-1 flex-wrap">
              按住
              {shortcutParts.map((part, index) => (
                <React.Fragment key={index}>
                  {index > 0 && <span>+</span>}
                  <kbd className="px-2 py-0.5 text-xs font-semibold text-gray-800 bg-white border border-gray-300 rounded">
                    {part.symbol} {part.name}
                  </kbd>
                </React.Fragment>
              ))}
              并说话
            </p>
            <p className="text-xs text-gray-500 mt-1">
              当前: {settings.shortcut}
            </p>
          </div>
          <Button variant="secondary" size="sm">
            更改
          </Button>
        </div>
      </div>

      {/* 麦克风 */}
      <div className="p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <h4 className="font-medium text-gray-900 mb-1">麦克风</h4>
            <p className="text-sm text-gray-600">
              {settings.microphone === 'auto' ? '自动检测' : settings.microphone}
            </p>
          </div>
          <Button variant="secondary" size="sm">
            更改
          </Button>
        </div>
      </div>

      {/* 语言 */}
      <div className="p-4 bg-gray-50 rounded-lg">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <h4 className="font-medium text-gray-900 mb-1">语言</h4>
            <p className="text-sm text-gray-600">
              {settings.language === 'zh' ? '中文(简体)' : settings.language === 'en' ? '英语' : settings.language}
            </p>
          </div>
          <Button variant="secondary" size="sm" onClick={openLanguageSelector}>
            更改
          </Button>
        </div>
      </div>
    </div>
  );
};

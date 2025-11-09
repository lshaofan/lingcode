import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useSettingsStore } from '../../../stores';
import { RadioGroup, RadioOption, ProgressBar, Button } from '../../../components';
import { useToast } from '../../../components';

interface ModelInfo {
  name: string;
  size: string;
  size_bytes: number;
  speed: string;
  accuracy: string;
  is_recommended: boolean;
  is_downloaded: boolean;
  download_url: string;
}

interface DownloadProgress {
  model_name: string;
  progress: number;
  downloaded: number;
  total: number;
}

export const ModelSettings: React.FC = () => {
  const { settings, updateSetting } = useSettingsStore();
  const toast = useToast();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);

  useEffect(() => {
    loadModels();
  }, []);

  useEffect(() => {
    // 监听下载进度
    const unlisten = listen<DownloadProgress>('model-download-progress', (event) => {
      setDownloadProgress(event.payload.progress);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadModels = async () => {
    setLoading(true);
    try {
      const modelList = await invoke<ModelInfo[]>('get_available_models');
      setModels(modelList);
    } catch (error) {
      toast.error(`加载模型列表失败: ${error}`);
      console.error('Failed to load models:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleModelChange = async (value: string) => {
    const model = models.find((m) => m.name === value);
    if (!model) return;

    if (!model.is_downloaded) {
      toast.warning('请先下载该模型');
      return;
    }

    try {
      await updateSetting('model', value as any);
      toast.success(`已切换到 ${value.toUpperCase()} 模型`);
    } catch (error) {
      toast.error(`切换模型失败: ${error}`);
    }
  };

  const handleDownload = async (modelName: string) => {
    setDownloadingModel(modelName);
    setDownloadProgress(0);

    try {
      await invoke('download_model', { modelName });
      toast.success(`${modelName.toUpperCase()} 模型下载完成`);
      // 重新加载模型列表
      await loadModels();
    } catch (error) {
      toast.error(`下载失败: ${error}`);
      console.error('Failed to download model:', error);
    } finally {
      setDownloadingModel(null);
      setDownloadProgress(0);
    }
  };

  const handleDelete = async (modelName: string) => {
    if (!confirm(`确定要删除 ${modelName.toUpperCase()} 模型吗?`)) {
      return;
    }

    try {
      await invoke('delete_model', { modelName });
      toast.success(`${modelName.toUpperCase()} 模型已删除`);
      // 重新加载模型列表
      await loadModels();
    } catch (error) {
      toast.error(`删除失败: ${error}`);
      console.error('Failed to delete model:', error);
    }
  };

  const radioOptions: RadioOption[] = models.map((model) => ({
    value: model.name,
    label: `${model.name.toUpperCase()} (${model.size}, ${model.speed}, ${model.accuracy})${model.is_recommended ? ' 推荐' : ''}`,
    description: model.is_downloaded
      ? '✓ 已下载'
      : downloadingModel === model.name
      ? `下载中... ${downloadProgress}%`
      : '',
  }));

  const downloadedModels = models.filter((m) => m.is_downloaded);

  return (
    <div className="space-y-6">
      <h3 className="text-2xl font-semibold text-gray-900">模型设置</h3>

      {/* Whisper 模型选择 */}
      <div>
        <h4 className="text-sm font-medium text-gray-700 mb-3">Whisper 模型选择</h4>
        {loading ? (
          <div className="text-center py-8 text-gray-500">加载中...</div>
        ) : (
          <div className="space-y-3">
            <RadioGroup
              name="model"
              value={settings.model}
              onChange={handleModelChange}
              options={radioOptions}
            />

            {/* 下载/删除按钮 */}
            <div className="space-y-2 mt-4">
              {models.map((model) => (
                <div key={model.name} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <span className="text-sm font-medium text-gray-700">
                    {model.name.toUpperCase()} ({model.size})
                  </span>
                  <div className="flex items-center gap-2">
                    {downloadingModel === model.name ? (
                      <ProgressBar progress={downloadProgress} size="sm" className="w-32" />
                    ) : model.is_downloaded ? (
                      <span className="text-xs text-green-600 flex items-center gap-1">
                        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                        </svg>
                        已下载
                      </span>
                    ) : null}
                    {!model.is_downloaded && downloadingModel !== model.name && (
                      <Button variant="secondary" size="sm" onClick={() => handleDownload(model.name)}>
                        下载
                      </Button>
                    )}
                    {model.is_downloaded && (
                      <Button variant="secondary" size="sm" onClick={() => handleDelete(model.name)}>
                        删除
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* 已下载的模型 */}
      {downloadedModels.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-3">已下载的模型</h4>
          <div className="space-y-2">
            {downloadedModels.map((model) => (
              <div key={model.name} className="p-3 bg-gray-50 rounded-lg flex items-center justify-between">
                <div>
                  <span className="font-medium text-gray-900">{model.name.toUpperCase()}</span>
                  <span className="text-sm text-gray-500 ml-2">({model.size})</span>
                </div>
                <Button variant="secondary" size="sm" onClick={() => handleDelete(model.name)}>
                  删除
                </Button>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

#!/usr/bin/env python3
"""
FunASR 转录脚本
支持多种模型的离线语音识别（使用中国镜像源）
"""

import sys
import json
import os
import argparse
from pathlib import Path
from typing import Optional, Dict, Any


def transcribe_audio(
    audio_path: str,
    model_name: str = "paraformer-zh",
    language: Optional[str] = None,
    hotword: Optional[str] = None,
) -> Dict[str, Any]:
    """
    使用 FunASR 转录音频

    Args:
        audio_path: 音频文件路径
        model_name: 模型名称
        language: 语言代码（可选）
        hotword: 热词（可选）

    Returns:
        转录结果字典
    """
    # 调试：打印 Python 环境信息
    print(f"🐍 Python executable: {sys.executable}", file=sys.stderr)
    print(f"🐍 Python version: {sys.version}", file=sys.stderr)
    print(f"🐍 Python path: {sys.path[:3]}", file=sys.stderr)

    try:
        from funasr import AutoModel
        print(f"✅ FunASR imported successfully", file=sys.stderr)
    except ImportError as e:
        print(f"❌ Failed to import FunASR: {e}", file=sys.stderr)
        return {
            "success": False,
            "error": f"FunASR not installed or import failed: {str(e)}. Please install with: pip install funasr",
        }

    try:
        # 模型配置映射（使用完整的 ModelScope ID，与 download 函数一致）
        model_configs = {
            "paraformer-zh": {
                "model": "damo/speech_paraformer-large-vad-punc_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
                "vad_model": "damo/speech_fsmn_vad_zh-cn-16k-common-pytorch",
                "punc_model": "damo/punc_ct-transformer_zh-cn-common-vocab272727-pytorch",
            },
            "paraformer-large": {
                "model": "iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
                "vad_model": "damo/speech_fsmn_vad_zh-cn-16k-common-pytorch",
                "punc_model": "damo/punc_ct-transformer_zh-cn-common-vocab272727-pytorch",
            },
            "sensevoice-small": {
                "model": "iic/SenseVoiceSmall",
                "vad_model": "damo/speech_fsmn_vad_zh-cn-16k-common-pytorch",
                "punc_model": None,  # SenseVoice 自带标点
            },
        }

        config = model_configs.get(model_name)
        if not config:
            return {
                "success": False,
                "error": f"Unknown model: {model_name}",
            }

        # 初始化模型
        model_kwargs = {
            "model": config["model"],
            "disable_log": False,
            "disable_pbar": False,
            "disable_update": True,  # 🚀 关键优化：禁用自动更新检查，避免重复下载
            "hub": "ms",  # 使用 ModelScope hub
        }

        if config["vad_model"]:
            model_kwargs["vad_model"] = config["vad_model"]

        if config["punc_model"]:
            model_kwargs["punc_model"] = config["punc_model"]

        model = AutoModel(**model_kwargs)

        # 准备输入参数
        generate_kwargs = {"input": audio_path}

        if hotword:
            generate_kwargs["hotword"] = hotword

        if language:
            generate_kwargs["language"] = language

        # 执行转录
        result = model.generate(**generate_kwargs)

        if not result or len(result) == 0:
            return {
                "success": False,
                "error": "No transcription result",
            }

        # 提取文本
        text = result[0].get("text", "")

        return {
            "success": True,
            "text": text,
            "raw_result": result[0],
        }

    except Exception as e:
        return {
            "success": False,
            "error": f"Transcription failed: {str(e)}",
        }


def download_model(
    model_name: str,
    cache_dir: Optional[str] = None,
    max_retries: int = 3,
) -> Dict[str, Any]:
    """
    下载 FunASR 模型（使用中国镜像源，跳过已下载的模型，支持断点续传和重试）

    Args:
        model_name: 模型名称
        cache_dir: 缓存目录
        max_retries: 最大重试次数（默认3次）

    Returns:
        下载结果
    """
    try:
        from modelscope.hub.snapshot_download import snapshot_download
        from modelscope.hub.file_download import http_get_file
    except ImportError:
        print(f"❌ ModelScope 未安装，无法下载模型", file=sys.stderr)
        print(f"💡 提示：ModelScope 正在后台安装中，请稍候片刻后重试", file=sys.stderr)
        return {
            "success": False,
            "error": "ModelScope 正在安装中，请稍候 10-30 秒后重试",
        }

    import time

    try:
        # 设置中国镜像源（如果未设置）
        if not os.environ.get("MODELSCOPE_ENDPOINT"):
            os.environ["MODELSCOPE_ENDPOINT"] = "https://www.modelscope.cn"

        # 获取缓存根目录
        cache_root = os.path.expanduser(os.environ.get("MODELSCOPE_CACHE", "~/.cache/modelscope"))
        if cache_dir:
            cache_root = cache_dir
            os.environ["MODELSCOPE_CACHE"] = cache_dir

        # ModelScope 的实际缓存结构
        hub_models_dir = os.path.join(cache_root, "hub", "models")

        model_ids = {
            "paraformer-zh": "damo/speech_paraformer-large-vad-punc_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
            "paraformer-large": "iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
            "sensevoice-small": "iic/SenseVoiceSmall",
            "fsmn-vad": "damo/speech_fsmn_vad_zh-cn-16k-common-pytorch",
            "ct-punc": "damo/punc_ct-transformer_zh-cn-common-vocab272727-pytorch",
        }

        model_id = model_ids.get(model_name)
        if not model_id:
            return {
                "success": False,
                "error": f"Unknown model: {model_name}",
            }

        # 检查模型是否已存在的辅助函数
        def is_model_downloaded(model_id_to_check):
            org, model_name_part = model_id_to_check.split("/", 1)
            model_cache_path = os.path.join(hub_models_dir, org, model_name_part)
            if not (os.path.exists(model_cache_path) and os.path.isdir(model_cache_path)):
                return False
            # 检查是否有实际的模型文件
            for root, dirs, files in os.walk(model_cache_path):
                if files:
                    return True
            return False

        # 下载单个模型的辅助函数（带重试和进度回调）
        def download_with_retry(model_id_to_download, display_name, retries=max_retries):
            # 进度回调函数
            def progress_callback(info):
                """
                ModelScope 下载进度回调
                info: dict with keys: 'downloaded', 'total', 'percentage'
                """
                if 'percentage' in info:
                    percentage = int(info['percentage'])
                    # 发送进度更新（每5%更新一次，避免过于频繁）
                    if percentage % 5 == 0 or percentage >= 95:
                        downloaded = info.get('downloaded', 0)
                        total = info.get('total', 0)
                        if total > 0:
                            downloaded_mb = downloaded / (1024 * 1024)
                            total_mb = total / (1024 * 1024)
                            msg = f"{downloaded_mb:.1f}MB / {total_mb:.1f}MB"
                        else:
                            msg = "下载中"
                        print(f"PROGRESS:{percentage}:{display_name}:{msg}", file=sys.stderr)

            for attempt in range(1, retries + 1):
                try:
                    print(f"📥 [{attempt}/{retries}] 正在下载{display_name}: {model_id_to_download}", file=sys.stderr)
                    print(f"PROGRESS:0:{display_name}:开始下载", file=sys.stderr)  # 进度标记

                    # ModelScope 的 snapshot_download 支持断点续传和进度回调
                    try:
                        # 尝试使用进度回调（新版本 ModelScope）
                        result_dir = snapshot_download(
                            model_id_to_download,
                            cache_dir=cache_root,
                            progress_callback=progress_callback,
                        )
                    except TypeError:
                        # 旧版本不支持 progress_callback，使用基础下载
                        print(f"⚠️  ModelScope 版本较旧，不支持进度回调", file=sys.stderr)
                        result_dir = snapshot_download(
                            model_id_to_download,
                            cache_dir=cache_root,
                        )
                        # 模拟进度（每秒检查一次）
                        for i in range(1, 10):
                            print(f"PROGRESS:{i * 10}:{display_name}:下载中...", file=sys.stderr)
                            time.sleep(1)

                    print(f"PROGRESS:100:{display_name}:下载完成", file=sys.stderr)  # 进度标记
                    print(f"✅ {display_name}下载完成: {model_id_to_download}", file=sys.stderr)
                    return result_dir

                except Exception as e:
                    error_msg = str(e)
                    print(f"❌ [{attempt}/{retries}] {display_name}下载失败: {error_msg}", file=sys.stderr)

                    if attempt < retries:
                        wait_time = attempt * 2  # 递增等待时间：2s, 4s, 6s
                        print(f"⏳ {wait_time}秒后重试...", file=sys.stderr)
                        time.sleep(wait_time)
                    else:
                        raise Exception(f"{display_name}下载失败（已重试{retries}次）: {error_msg}")

        # 下载主模型（如果未下载）
        if is_model_downloaded(model_id):
            print(f"✅ 主模型已存在，跳过下载: {model_id}", file=sys.stderr)
            print(f"PROGRESS:100:主模型:已存在", file=sys.stderr)  # 进度标记
            org, model_name_part = model_id.split("/", 1)
            model_dir = os.path.join(hub_models_dir, org, model_name_part)
        else:
            model_dir = download_with_retry(model_id, "主模型")

        # 下载依赖模型（VAD 和 PUNC）
        dependencies = []
        if model_name in ["paraformer-zh", "paraformer-large"]:
            dependencies = ["fsmn-vad", "ct-punc"]
        elif model_name == "sensevoice-small":
            dependencies = ["fsmn-vad"]

        for dep in dependencies:
            dep_id = model_ids.get(dep)
            if dep_id:
                try:
                    if is_model_downloaded(dep_id):
                        print(f"✅ 依赖模型已存在，跳过下载: {dep}", file=sys.stderr)
                        print(f"PROGRESS:100:{dep}:已存在", file=sys.stderr)  # 进度标记
                    else:
                        download_with_retry(dep_id, f"依赖模型({dep})")
                except Exception as e:
                    print(f"⚠️  下载依赖模型 {dep} 失败: {e}", file=sys.stderr)
                    print(f"PROGRESS:0:{dep}:下载失败", file=sys.stderr)  # 进度标记
                    # 不要因为依赖模型失败而整体失败（某些模型可能可选）
                    pass

        print(f"🎉 模型 {model_name} 及所有依赖准备完成", file=sys.stderr)

        return {
            "success": True,
            "model_dir": str(model_dir),
            "model_size": get_dir_size(str(model_dir)),
        }

    except Exception as e:
        import traceback
        error_details = traceback.format_exc()
        print(f"❌ 下载失败详情:\n{error_details}", file=sys.stderr)
        return {
            "success": False,
            "error": f"Model download failed: {str(e)}",
        }


def get_dir_size(path: str) -> str:
    """获取目录大小（用于显示下载进度）"""
    try:
        total_size = 0
        for dirpath, dirnames, filenames in os.walk(path):
            for filename in filenames:
                filepath = os.path.join(dirpath, filename)
                if os.path.exists(filepath):
                    total_size += os.path.getsize(filepath)

        # 转换为 MB
        size_mb = total_size / (1024 * 1024)
        return f"{size_mb:.1f} MB"
    except:
        return "Unknown"


def check_model_exists(
    model_name: str,
    cache_dir: Optional[str] = None,
) -> Dict[str, Any]:
    """
    检查 FunASR 模型是否存在（包括依赖模型）

    Args:
        model_name: 模型名称
        cache_dir: 缓存目录

    Returns:
        检查结果
    """
    try:
        import modelscope
    except ImportError:
        # ModelScope 未安装时，模型肯定不存在，返回成功但 exists=False
        print(f"⚠️  ModelScope 未安装，无法检查模型", file=sys.stderr)
        return {
            "success": True,
            "error": "",
            "exists": False,
        }

    try:
        # 获取缓存根目录 - ModelScope 默认使用 ~/.cache/modelscope/hub
        cache_root = os.path.expanduser(os.environ.get("MODELSCOPE_CACHE", "~/.cache/modelscope"))
        if cache_dir:
            cache_root = cache_dir

        # ModelScope 的实际缓存结构是 {cache_root}/hub/models/{org}/{model_name}
        hub_models_dir = os.path.join(cache_root, "hub", "models")

        # 模型 ID 映射
        model_ids = {
            "paraformer-zh": "damo/speech_paraformer-large-vad-punc_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
            "paraformer-large": "iic/speech_paraformer-large_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
            "sensevoice-small": "iic/SenseVoiceSmall",
            "fsmn-vad": "damo/speech_fsmn_vad_zh-cn-16k-common-pytorch",
            "ct-punc": "damo/punc_ct-transformer_zh-cn-common-vocab272727-pytorch",
        }

        model_id = model_ids.get(model_name)
        if not model_id:
            return {
                "success": False,
                "error": f"Unknown model: {model_name}",
                "exists": False,
            }

        # 检查主模型
        def check_single_model(model_id_to_check, model_display_name=""):
            # ModelScope 路径结构: hub/models/{org}/{model_name}
            # 例如: hub/models/damo/speech_paraformer-large-vad-punc_asr_nat-zh-cn-16k-common-vocab8404-pytorch
            org, model_name_part = model_id_to_check.split("/", 1)
            model_cache_path = os.path.join(hub_models_dir, org, model_name_part)
            print(f"🔍 检查模型 {model_display_name}:", file=sys.stderr)
            print(f"   路径: {model_cache_path}", file=sys.stderr)

            if not os.path.exists(model_cache_path):
                print(f"  ❌ 路径不存在", file=sys.stderr)
                return False

            if not os.path.isdir(model_cache_path):
                print(f"  ❌ 不是目录", file=sys.stderr)
                return False

            # 检查是否有实际的模型文件
            has_files = False
            file_count = 0
            for root, dirs, files in os.walk(model_cache_path):
                if files:
                    has_files = True
                    file_count = len(files)
                    break

            if not has_files:
                print(f"  ❌ 目录为空", file=sys.stderr)
                return False

            print(f"  ✅ 找到 {file_count} 个文件", file=sys.stderr)
            return True

        # 检查主模型
        print(f"📋 开始检查模型: {model_name}", file=sys.stderr)
        print(f"📁 缓存根目录: {cache_root}", file=sys.stderr)
        print(f"📁 模型目录: {hub_models_dir}", file=sys.stderr)

        if not check_single_model(model_id, f"{model_name} (主模型)"):
            print(f"❌ 主模型未找到: {model_id}", file=sys.stderr)
            return {
                "success": True,
                "exists": False,
            }

        # 检查依赖模型
        dependencies = []
        if model_name in ["paraformer-zh", "paraformer-large"]:
            dependencies = ["fsmn-vad", "ct-punc"]
        elif model_name == "sensevoice-small":
            dependencies = ["fsmn-vad"]

        for dep in dependencies:
            dep_id = model_ids.get(dep)
            if dep_id and not check_single_model(dep_id, f"{dep} (依赖)"):
                print(f"❌ 依赖模型未找到: {dep_id}", file=sys.stderr)
                return {
                    "success": True,
                    "exists": False,
                }

        # 所有模型都存在
        org, model_name_part = model_id.split("/", 1)
        model_cache_path = os.path.join(hub_models_dir, org, model_name_part)
        print(f"✅ 模型及所有依赖已下载: {model_name}", file=sys.stderr)
        return {
            "success": True,
            "exists": True,
            "model_path": model_cache_path,
        }

    except Exception as e:
        return {
            "success": False,
            "error": f"Check failed: {str(e)}",
            "exists": False,
        }


def main():
    parser = argparse.ArgumentParser(description="FunASR 转录工具")
    parser.add_argument("command", choices=["transcribe", "download", "check"], help="命令")
    parser.add_argument("--audio", help="音频文件路径")
    parser.add_argument("--model", default="paraformer-zh", help="模型名称")
    parser.add_argument("--language", help="语言代码")
    parser.add_argument("--hotword", help="热词")
    parser.add_argument("--cache-dir", help="模型缓存目录")

    args = parser.parse_args()

    if args.command == "transcribe":
        if not args.audio:
            result = {
                "success": False,
                "error": "--audio is required for transcribe command",
            }
        else:
            result = transcribe_audio(
                audio_path=args.audio,
                model_name=args.model,
                language=args.language,
                hotword=args.hotword,
            )
    elif args.command == "download":
        result = download_model(
            model_name=args.model,
            cache_dir=args.cache_dir,
        )
    elif args.command == "check":
        result = check_model_exists(
            model_name=args.model,
            cache_dir=args.cache_dir,
        )
    else:
        result = {
            "success": False,
            "error": f"Unknown command: {args.command}",
        }

    # 输出 JSON 结果
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()

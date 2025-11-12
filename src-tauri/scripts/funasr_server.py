#!/usr/bin/env python3
"""
FunASR 常驻服务
保持 Python 进程运行，模型只加载一次，通过 stdin/stdout 进行 JSON-RPC 通信
"""

import sys
import json
import os
from pathlib import Path
from typing import Optional, Dict, Any
from contextlib import contextmanager

@contextmanager
def suppress_stdout():
    """临时重定向 stdout 到 stderr，防止 FunASR 输出污染 JSON 响应"""
    original_stdout = sys.stdout
    try:
        sys.stdout = sys.stderr
        yield
    finally:
        sys.stdout = original_stdout

# 全局模型缓存
_model_cache = {}


def load_model(model_name: str) -> Any:
    """加载或获取缓存的模型"""
    if model_name in _model_cache:
        print(f"♻️  Using cached model: {model_name}", file=sys.stderr)
        return _model_cache[model_name]

    print(f"📦 Loading model: {model_name}", file=sys.stderr)

    try:
        from funasr import AutoModel
    except ImportError as e:
        raise ImportError(f"FunASR not installed: {e}")

    # 模型配置
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
            "punc_model": None,
        },
    }

    config = model_configs.get(model_name)
    if not config:
        raise ValueError(f"Unknown model: {model_name}")

    # 初始化模型
    model_kwargs = {
        "model": config["model"],
        "disable_log": True,
        "disable_pbar": True,
        "disable_update": True,
        "hub": "ms",
    }

    if config["vad_model"]:
        model_kwargs["vad_model"] = config["vad_model"]

    if config["punc_model"]:
        model_kwargs["punc_model"] = config["punc_model"]

    # 使用 suppress_stdout 防止 FunASR 输出污染 JSON 响应
    with suppress_stdout():
        model = AutoModel(**model_kwargs)

    _model_cache[model_name] = model

    print(f"✅ Model loaded and cached: {model_name}", file=sys.stderr)
    return model


def transcribe_audio(
    audio_path: str,
    model_name: str = "paraformer-zh",
    language: Optional[str] = None,
    hotword: Optional[str] = None,
) -> Dict[str, Any]:
    """转录音频"""
    try:
        # 验证音频文件
        if not os.path.exists(audio_path):
            return {
                "success": False,
                "error": f"Audio file not found: {audio_path}",
            }

        # 检查音频文件大小（至少 1KB）
        file_size = os.path.getsize(audio_path)
        print(f"📊 Audio file size: {file_size} bytes", file=sys.stderr)

        if file_size < 1000:
            return {
                "success": False,
                "error": f"Audio file too small: {file_size} bytes (minimum 1000 bytes)",
            }

        model = load_model(model_name)

        # 准备输入参数
        generate_kwargs = {"input": audio_path}

        if hotword:
            generate_kwargs["hotword"] = hotword

        if language:
            generate_kwargs["language"] = language

        # 执行转录（使用 suppress_stdout 防止输出污染）
        print(f"🎤 Starting transcription...", file=sys.stderr)
        with suppress_stdout():
            result = model.generate(**generate_kwargs)
        print(f"✅ Transcription completed", file=sys.stderr)
        print(f"📊 Raw result type: {type(result)}", file=sys.stderr)
        print(f"📊 Raw result length: {len(result) if result else 0}", file=sys.stderr)
        if result and len(result) > 0:
            print(f"📊 First result: {result[0]}", file=sys.stderr)

        if not result or len(result) == 0:
            return {
                "success": False,
                "error": "No transcription result (empty result from model)",
            }

        # 提取文本
        text = result[0].get("text", "")

        # 如果文本为空，返回特殊消息
        if not text or text.strip() == "":
            return {
                "success": True,
                "text": "",  # 空文本也算成功，可能是静音
            }

        return {
            "success": True,
            "text": text,
        }

    except RuntimeError as e:
        error_msg = str(e)
        if "stack expects a non-empty TensorList" in error_msg:
            return {
                "success": False,
                "error": "音频太短或为静音，无法识别。请录制更长的音频（至少1秒）。",
            }
        else:
            import traceback
            error_details = traceback.format_exc()
            print(f"❌ Runtime error:\\n{error_details}", file=sys.stderr)
            return {
                "success": False,
                "error": f"转录失败: {error_msg}",
            }
    except Exception as e:
        import traceback
        error_details = traceback.format_exc()
        print(f"❌ Transcription error:\\n{error_details}", file=sys.stderr)
        return {
            "success": False,
            "error": f"转录失败: {str(e)}",
        }


def handle_request(request: Dict[str, Any]) -> Dict[str, Any]:
    """处理单个请求"""
    method = request.get("method")
    params = request.get("params", {})

    if method == "transcribe":
        return transcribe_audio(
            audio_path=params.get("audio_path"),
            model_name=params.get("model_name", "paraformer-zh"),
            language=params.get("language"),
            hotword=params.get("hotword"),
        )
    elif method == "ping":
        return {"success": True, "message": "pong"}
    elif method == "shutdown":
        return {"success": True, "message": "shutting down"}
    else:
        return {
            "success": False,
            "error": f"Unknown method: {method}",
        }


def main():
    """主循环：读取 stdin 的 JSON 请求，输出 JSON 响应"""
    print("🚀 FunASR Server started", file=sys.stderr)
    print(f"🐍 Python: {sys.executable}", file=sys.stderr)
    print(f"🐍 Version: {sys.version}", file=sys.stderr)

    # 确保 stdout 立即刷新
    sys.stdout.reconfigure(line_buffering=True)

    while True:
        try:
            # 读取一行 JSON 请求
            line = sys.stdin.readline()
            if not line:
                break

            line = line.strip()
            if not line:
                continue

            # 解析请求
            request = json.loads(line)
            print(f"📨 Received request: {request.get('method')}", file=sys.stderr)

            # 处理请求
            response = handle_request(request)

            # 输出响应（单行 JSON）
            print(json.dumps(response, ensure_ascii=False), flush=True)

            # 如果是 shutdown 命令，退出
            if request.get("method") == "shutdown":
                break

        except json.JSONDecodeError as e:
            error_response = {
                "success": False,
                "error": f"Invalid JSON: {str(e)}",
            }
            print(json.dumps(error_response, ensure_ascii=False), flush=True)
        except Exception as e:
            import traceback
            error_details = traceback.format_exc()
            print(f"❌ Error processing request:\\n{error_details}", file=sys.stderr)
            error_response = {
                "success": False,
                "error": f"Server error: {str(e)}",
            }
            print(json.dumps(error_response, ensure_ascii=False), flush=True)

    print("👋 FunASR Server stopped", file=sys.stderr)


if __name__ == "__main__":
    main()

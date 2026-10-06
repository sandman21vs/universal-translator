"""Bundle CPU inference libraries, excluding optional CUDA/cuDNN binaries."""
from PyInstaller.utils.hooks import collect_data_files, collect_dynamic_libs

def cpu_library(path):
    name = path.lower()
    return not any(part in name for part in ("cudnn", "cublas", "cudart", "nccl"))

binaries = [item for item in collect_dynamic_libs("ctranslate2") if cpu_library(item[0])]
datas = collect_data_files("ctranslate2", excludes=["**/cudnn*", "**/cublas*", "**/cudart*", "**/nccl*"])

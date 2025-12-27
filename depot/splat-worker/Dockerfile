# Gaussian Splatting Worker
# Requires NVIDIA GPU runtime (nvidia-docker2)

FROM nvidia/cuda:12.1-devel-ubuntu22.04

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    python3-dev \
    git \
    wget \
    libgl1-mesa-glx \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender1 \
    && rm -rf /var/lib/apt/lists/*

# Set up Python
RUN ln -s /usr/bin/python3 /usr/bin/python

WORKDIR /app

# Install PyTorch with CUDA support first
RUN pip install --no-cache-dir torch torchvision --index-url https://download.pytorch.org/whl/cu121

# Install other dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy worker code
COPY src/ ./src/

# Environment
ENV PYTHONUNBUFFERED=1
ENV JOBS_DIR=/data/jobs
ENV MAPS_DIR=/data/maps
ENV SESSIONS_DIR=/data/sessions

VOLUME ["/data/jobs", "/data/maps", "/data/sessions"]

CMD ["python", "src/worker.py"]

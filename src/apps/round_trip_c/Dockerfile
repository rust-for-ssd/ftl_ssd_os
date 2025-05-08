FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    xz-utils \
    make \
    libmpc3 \
    binutils \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Set environment variables
ENV TOOLCHAIN_VERSION=2025.05.01-nightly
ENV TOOLCHAIN_FILENAME=riscv32-elf-ubuntu-22.04-llvm-nightly-${TOOLCHAIN_VERSION}.tar.xz
ENV TOOLCHAIN_URL=https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2025.05.01/riscv32-elf-ubuntu-22.04-llvm-nightly-2025.05.01-nightly.tar.xz

# Download first, then extract
RUN curl -L -o /tmp/${TOOLCHAIN_FILENAME} ${TOOLCHAIN_URL} && \
    tar -xJf /tmp/${TOOLCHAIN_FILENAME} -C /opt && \
    rm /tmp/${TOOLCHAIN_FILENAME}

# Add riscv toolchain to PATH
ENV PATH="/opt/riscv/bin:$PATH"

# Create working directory
WORKDIR /app

# Copy project files
COPY . .

CMD [ "bash" ]
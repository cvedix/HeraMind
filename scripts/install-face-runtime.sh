#!/usr/bin/env bash
# Install the already-built runtime and downloaded face models on this host.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "$EUID" -ne 0 ]]; then
  echo "Chạy bằng sudo để cập nhật dịch vụ HeraMind." >&2
  exit 1
fi
for relative in target/debug/heramind target/debug/heramind-extension-runner \
  .local/face-models/det_10g.onnx .local/face-models/w600k_r50.onnx web/dist/index.html; do
  [[ -s "$project_dir/$relative" ]] || { echo "Thiếu tệp đã build/tải: $relative" >&2; exit 1; }
done
model_dir=/var/lib/heramind/extensions/face-recognition/models
install -d -o heramind -g heramind "$model_dir"
for model in det_10g.onnx w600k_r50.onnx; do
  # Preserve an existing model rather than overwriting a custom deployment.
  if [[ ! -s "$model_dir/$model" ]]; then
    install -o heramind -g heramind -m 644 "$project_dir/.local/face-models/$model" "$model_dir/$model"
  fi
done
for binary in heramind heramind-extension-runner; do
  install -m 755 "$project_dir/target/debug/$binary" "/usr/local/bin/$binary.face-update"
  mv -f "/usr/local/bin/$binary.face-update" "/usr/local/bin/$binary"
done
install -d /var/www/heramind
cp -a "$project_dir/web/dist/." /var/www/heramind/
systemctl restart heramind
systemctl is-active heramind
echo "Đã cập nhật runtime và mô hình. Tiếp tục tạo dashboard --with-faces và chạy rtsp-face-feed.py."

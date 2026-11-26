# PaddleOCR-VL cục bộ cho HeraMind

Thiết lập này chạy PaddleOCR-VL 1.6 trên CPU của macOS Apple Silicon. Extension
`paddle-ocr-vl` gọi dịch vụ HTTP cục bộ; dashboard dùng ảnh tải lên thủ công.
Quá trình OCR không cần backend chat hoặc khóa AgentRouter.

## Khởi động

Từ thư mục gốc HeraMind:

```bash
bash scripts/paddle-ocr-vl-service.sh setup
bash scripts/paddle-ocr-vl-service.sh download
bash scripts/paddle-ocr-vl-service.sh start
```

`setup` tạo Python 3.11.18 và môi trường riêng trong
`data/paddle-ocr-vl-service/`. `download` tải và kiểm tra nạp model;
`start` giữ model trong bộ nhớ rồi phục vụ tại `http://127.0.0.1:8000`.
Giữ tiến trình này chạy cùng HeraMind. CPU cần thêm thời gian cho ảnh lớn
hoặc tài liệu nhiều chữ.

```bash
bash scripts/paddle-ocr-vl-service.sh status
```

Khi sẵn sàng, phản hồi có `status: "ok"`, `model_loaded: true` và
`load_error: null`. Kết quả này chỉ xác nhận model đã nạp; kiểm tra OCR
bằng ảnh thật để xác nhận suy luận.

## Cấu hình extension

Cài `paddle-ocr-vl` từ marketplace, sau đó:

```bash
./target/debug/heramind extension config paddle-ocr-vl --set '{"endpoint":"http://127.0.0.1:8000","language":"multilingual","timeout_ms":120000,"use_doc_orientation_classify":false,"use_doc_unwarping":false}'
```

Ngôn ngữ là gợi ý cho extension; VLM xử lý nội dung đa ngôn ngữ. Bật xoay
ảnh hoặc làm phẳng tài liệu sẽ yêu cầu nạp thêm thành phần tiền xử lý.

## Dashboard OCR

`dashboard.json` là mẫu cho dashboard **OCR tài liệu**, gồm:

- Widget `paddle-ocr-vl-card` để tải ảnh và xem văn bản/bảng.
- Bốn thẻ lấy chỉ số thật từ extension: lượt yêu cầu, thành công, thất bại
  và thời gian xử lý gần nhất.
- Hướng dẫn sử dụng bằng tiếng Việt.

Tạo một dashboard mới từ mẫu, không thay thế dashboard hiện có:

```bash
python3 - <<'PY'
import json
import subprocess
from pathlib import Path

template = json.loads(Path('deploy/paddle-ocr-vl/dashboard.json').read_text())
subprocess.run([
    './target/debug/heramind', 'dashboard', 'create',
    '--name', template['name'], '--description', template['description'],
    '--components', json.dumps(template['components'], ensure_ascii=False),
], check=True)
PY
```

Các thẻ hiển thị dấu gạch khi chưa có dữ liệu; không tạo số liệu giả.
Tải ảnh rõ nét rồi bấm **Run OCR**. Dùng **Text** cho văn bản và **Table**
cho bảng. Một lượt thử có ảnh mẫu sẽ được tính trong chỉ số của extension.
Điểm confidence trong dịch vụ upstream là giá trị tương thích cố định,
không phải xác suất chính xác do VLM cung cấp; cần đối chiếu kết quả với ảnh.

## Nguồn và dữ liệu

- Dịch vụ Python: [NeoMind-Extensions v2.14.0](https://github.com/camthink-ai/NeoMind-Extensions/tree/v2.14.0/extensions/paddle-ocr-vl/server).
  Launcher kiểm tra SHA-256 của `server.py` trước khi chạy.
- Model: `PaddlePaddle/PaddleOCR-VL-1.6` và `PaddlePaddle/PP-DocLayoutV3`.
  Cache hiện tại khoảng 1,9 GB, nằm dưới
  `data/paddle-ocr-vl-service/models/official_models/`.
- Các phiên bản Python package đã dùng nằm trong `requirements-macos.lock`.
- Dữ liệu, môi trường Python, ảnh kiểm thử và trọng số được Git bỏ qua.
- Với cấu hình này, ảnh được gửi tới dịch vụ loopback trên chính máy; chỉ
  việc tải model cần kết nối tới kho model.

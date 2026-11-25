# Tích hợp NeoMind v1.3.3 vào HeraMind

## Mốc đồng bộ

| Mốc | Commit |
| --- | --- |
| HeraMind trước cập nhật | `c350b89f55f069307a88533880217650e1d0d2aa` |
| Baseline NeoMind v0.9.24 | `4c41362ad785b203e20cf49cc2dd3adb325d453e` |
| Bản phát hành NeoMind v1.3.3 | `9c09e8759bdca7666ba8ae33c371d51870049073` |

Bản tích hợp đối chiếu ba cây mã nguồn sau khi ánh xạ namespace upstream sang HeraMind. Tại thời điểm đồng bộ, nhánh `main` của NeoMind có thêm commit `f29ef5fc` cập nhật ảnh README. HeraMind dùng hình ảnh mang nhận diện của dự án. [Bản phát hành NeoMind v1.3.3](https://github.com/camthink-ai/NeoMind/releases/tag/v1.3.3)

## Thay đổi trong bản tích hợp

- Giao diện di động và chat mới, nhắc đến thiết bị hoặc tác tử trong nội dung chat và cải tiến xử lý khi lịch sử vượt giới hạn ngữ cảnh.
- Cải tiến thông báo theo phán quyết của tác tử, theo dõi đầu vào ảnh và chẩn đoán MQTT, quy tắc và chuyển đổi dữ liệu.
- Phân trang telemetry theo offset thực tế trong kho dữ liệu và hỗ trợ cursor cho trang kế tiếp. CLI HeraMind tiếp tục hỗ trợ `device history --offset`, `--aggregate sum|count` và `dashboard inspect`.
- EventList được đăng ký trong dashboard, cấu hình widget và danh sách kiểu widget phía máy chủ. Các ví dụ HeraCam/bodycam và script dashboard AI của HeraMind tiếp tục dùng các điểm vào hiện có.
- Giữ logo, giao diện xanh, tiếng Việt mặc định và múi giờ `Asia/Ho_Chi_Minh` khi chưa có lựa chọn. Múi giờ đã lưu được giữ khi mở lại.
- Giữ crate và binary `heramind-*`, biến `HERAMIND_*`, app ID `com.heramind.heramind` và kênh cập nhật `cvedix/HeraMind`. URL marketplace và danh mục model tiếp tục trỏ đến các repository NeoMind tương ứng.
- SDK extension lên **0.8.1**, ABI vẫn là **3**. Trình nạp ưu tiên symbol `heramind_*` và hỗ trợ symbol `neomind_*` cho gói tương thích. SDK trình duyệt giữ cả hai namespace và tên token mà widget cũ dùng.

## Nâng cấp dữ liệu

Sao lưu thư mục dữ liệu và các khóa cấu hình liên quan sau khi dừng phiên bản đang chạy. Thử bản 1.3.3 trên bản sao dữ liệu trước khi thay binary vận hành. Khi quay về phiên bản cũ, dùng bản sao lưu được tạo trước nâng cấp cùng với binary cũ.

Các ghi chú về hành vi mới của tác tử, cấu hình và telemetry nằm trong [changelog](../CHANGELOG.md) và [tham khảo cấu hình](configuration.md).

## Kiểm chứng cục bộ

| Hạng mục | Kết quả |
| --- | --- |
| Bản dựng server, CLI và kiểm tra biên dịch desktop Linux | Đạt với các lockfile hiện tại |
| Kiểm thử đơn vị và tích hợp Rust | 4.615 kiểm thử đạt sau khi chạy lại các nhóm bị ảnh hưởng. 55 kiểm thử được đánh dấu bỏ qua |
| Ví dụ mã trong tài liệu Rust | 15 kiểm thử đạt, 16 được đánh dấu bỏ qua |
| Frontend | 600 kiểm thử đạt. TypeScript, ESLint, build Vite và kiểm tra độ tương phản WCAG AA đạt |
| Giao diện di động | 76 lượt kiểm tra trang và kích thước màn hình đạt. Luồng điều hướng qua menu chat, hub Hệ thống và các trang thiết bị/dashboard đạt |
| Smoke test trình duyệt và API | 11 bước đạt. OpenAPI có 294 đường dẫn và 152 schema, mọi tham chiếu nội bộ đều phân giải được |
| Nâng cấp từ 0.9.24 | Bản 1.3.3 mở bản sao dữ liệu mô phỏng được tạo bằng binary 0.9.24. Phiên đăng nhập, múi giờ đã lưu, thiết bị và EventList hoạt động. Tổng, số lượng và trang telemetry ở offset 110 khớp trên 115 điểm dữ liệu |
| Chuỗi nghiệp vụ MQTT và webhook | 15 bước đạt, gồm lưu telemetry, kích hoạt quy tắc, gửi thông báo và chuyển tiếp dữ liệu |
| Script và bộ dữ liệu đánh giá | 17 kiểm thử script HeraMind đạt. 440 ca đánh giá và kiểm tra tương ứng CLI đạt |

Có thể chạy lại smoke test bằng `cargo build -p heramind-cli --bin heramind --locked`, `npm --prefix web ci`, `npm --prefix web run build:check` và `node web/scripts/smoke-heramind-upgrade.cjs`. Script dùng thư mục dữ liệu tạm và Chromium của Playwright. Có thể chỉ định đường dẫn trình duyệt bằng `CHROME_PATH`.

`npm audit --omit=dev` còn 3 cảnh báo mức vừa trong chuỗi phụ thuộc i18next và Tailwind Typography. Đã cập nhật các phiên bản tương thích của Zod và source-map-js để xử lý cảnh báo mức cao trong tập phụ thuộc production.

Audit bao gồm công cụ phát triển còn 22 cảnh báo, trong đó 4 cảnh báo nghiêm trọng thuộc chuỗi Vitest. Các đề xuất xử lý nhóm Vitest cần nâng phiên bản lớn.

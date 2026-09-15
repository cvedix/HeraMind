# Tích hợp NeoMind v0.9.24 vào HeraMind

Ngày thực hiện: 16/09/2026. Nhánh: `codex/sync-neomind-v0.9.24`.

## Phạm vi và nguồn

| Mốc | Commit |
|---|---|
| HeraMind trước cập nhật, phiên bản 0.9.11 | `cbf789d3` |
| Baseline nội dung NeoMind v0.9.11 | `c7b54146518d3063f9a6b9aded7d87a0b43297a4` |
| NeoMind v0.9.24 được tích hợp | `4c41362ad785b203e20cf49cc2dd3adb325d453e` |

Nguồn: [release NeoMind v0.9.24](https://github.com/camthink-ai/NeoMind/releases/tag/v0.9.24), [commit mục tiêu](https://github.com/camthink-ai/NeoMind/commit/4c41362ad785b203e20cf49cc2dd3adb325d453e).

Hai repository không có tổ tiên Git chung. Bản cập nhật dùng ghép nội dung ba phiên bản: baseline NeoMind, HeraMind hiện tại và NeoMind mục tiêu. Namespace được ánh xạ sang HeraMind trước khi ghép; 39 trường hợp xung đột/đổi cấu trúc/nhị phân được xử lý riêng. [Báo cáo phân tích ban đầu](upstream-update-assessment-2026-09-16.md) ghi lại phép đối chiếu trước tích hợp.

Phiên bản workspace, frontend và desktop là **0.9.24**; extension SDK là **0.7.1**, ABI vẫn **3**. Đây là bản cập nhật mã nguồn trong workspace; chưa phát hành binary hoặc triển khai lên hệ thống vận hành.

## Các cập nhật chính

- Backend LLM cục bộ tích hợp và danh mục model; cải thiện xử lý context, thinking và các vòng gọi công cụ của agent.
- Kết nối ứng dụng nhắn tin, OpenAPI và giao diện tài liệu API.
- Lưu session bền vững, thu hồi session, giới hạn đăng nhập, bảo vệ đường dẫn extension, mã hóa cấu hình nhạy cảm và kiểm tra phiên bản schema dữ liệu.
- Backup, cải thiện storage, telemetry, quy tắc tự động và độ tin cậy khi xử lý sự kiện.
- Sidebar mới, quản lý model, bộ nhớ và các màn hình cấu hình mới.
- SDK extension 0.7.1, truyền dữ liệu nhị phân qua IPC và cập nhật runtime extension.

Các con số benchmark hoặc acceptance trong phần changelog upstream là kết quả do upstream công bố; không phải số đo riêng của HeraMind.

## Giữ bản sắc và tương thích HeraMind

| Thành phần | Kết quả tích hợp |
|---|---|
| Tên sản phẩm | HeraMind; crate/package/binary `heramind-*`; app ID `com.heramind.heramind` |
| Giao diện | Giữ logo PNG, nền Honeycomb và màu xanh; áp dụng màu xanh vào sidebar và trạng thái chọn mới |
| Ngôn ngữ | Tiếng Việt mặc định; thêm 436 chuỗi cho tính năng mới và sửa placeholder; bộ chọn hỗ trợ vi/en/zh |
| Dữ liệu/cấu hình | Giữ `HERAMIND_*`, thư mục dữ liệu và định danh ứng dụng hiện có |
| Nguồn cập nhật sản phẩm | `cvedix/HeraMind`; không chuyển updater sang NeoMind |
| Hệ sinh thái bên ngoài | Giữ tên thật của các repository `camthink-ai/NeoMind-Extensions`, `NeoMind-DeviceTypes`, `NeoMind-Dashboard-Components`, `NeoMind-Runtimes` |
| Extension native | Ưu tiên symbol `heramind_*`, hỗ trợ fallback `neomind_*`; vẫn kiểm tra ABI và các export bắt buộc |
| SDK trình duyệt | Giữ `window.heramind` / `HeraMindStream`, thêm alias `window.neomind` / `NeoMindStream` |
| HeraCam/bodycam | Giữ examples RV1126B và bodycam, truy vấn dữ liệu chỉ đọc, offset thời gian và so sánh hai khung thời gian |
| Dashboard | Đăng ký EventList trong registry mới; giữ truy vấn sum/count phía server đồng thời hỗ trợ pagination mới |
| Giới hạn model | Giữ `HERAMIND_MAX_CONTEXT` khi áp dụng kết quả dò capability Ollama |
| Múi giờ và onboarding | Dùng Asia/Ho_Chi_Minh khi chưa có lựa chọn; giữ nguyên múi giờ đã lưu; bỏ mục đăng ký bản tin CamThink khỏi luồng tạo tài khoản |

Không có nâng cấp ABI tự động cho extension cũ thiếu JSON bridge. Các extension như vậy vẫn cần build lại với SDK tương thích; fallback namespace không bỏ qua kiểm tra ABI.

## Kiểm chứng

| Kiểm tra | Kết quả |
|---|---|
| `cargo check --workspace --locked` | Đạt |
| `cargo test --workspace --locked -- --test-threads=1` | 3.158 test đạt, 0 lỗi, 62 ignored; 89 nhóm test/doc-test, dùng data dir tạm |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Đạt |
| `cargo fmt --all -- --check` | Đạt |
| `npm --prefix web test -- --run` | 202/202 test, 24 file |
| `npm --prefix web run lint:ci` | Đạt, không có lỗi ESLint |
| `npm --prefix web run build:check` | Đạt TypeScript và Vite; không có vòng phụ thuộc giữa các output chunk |
| `cargo check --manifest-path web/src-tauri/Cargo.toml --locked` | Đạt trên Linux, dùng chung thư mục target |
| `git diff --check`, JSON/TOML, ảnh README | Đạt |

Smoke test `node web/scripts/smoke-heramind-upgrade.cjs` **đạt 11 bước** trên bản debug 0.9.24 và frontend production:

- Server khởi động bằng dữ liệu tạm; API thiết bị từ chối truy cập thiếu xác thực.
- OpenAPI có 278 path; mọi `$ref` nội bộ đều phân giải được.
- Trình duyệt mới dùng tiếng Việt kể cả khi browser locale là tiếng Anh; cài đặt chưa chọn múi giờ dùng Asia/Ho_Chi_Minh.
- Đăng nhập qua giao diện, title HeraMind, theme sáng/tối màu xanh và hai namespace SDK trình duyệt hoạt động.
- Đăng ký device type HeraCam RV1126B nguyên bản, ghi ba metric 4/7/9 và đọc `count=3`, `sum=20`.
- CLI `device history --time-range 1h --offset 1h --aggregate sum` trả tổng 5 từ dữ liệu giờ trước, không trộn với dữ liệu giờ hiện tại.
- Lưu, tải lại và hiển thị widget EventList trên `/visual-dashboard/:id`.
- Các trang thiết bị, dashboard và cài đặt mở được, không có exception JavaScript.
- Sau khi dừng/chạy lại server, session hiện có vẫn truy cập được; dữ liệu HeraCam và dashboard vẫn tồn tại.

Lần chạy cuối dùng `/tmp/heramind-upgrade-smoke-JtpghH`; file `smoke-results.json`, ảnh và log nằm trong thư mục này. Những artifact tạm này không được đưa vào Git.

Các test mới kiểm tra đầy đủ khóa/placeholder tiếng Việt; phân biệt offset phân trang và aggregate; sử dụng giá trị sum/count do server tính; đăng ký EventList; ưu tiên/fallback symbol native SDK; và giữ múi giờ đã lưu qua lần mở lại storage.

Có thể chạy lại smoke test với:

```bash
cargo build -p heramind-cli --bin heramind --locked
npm --prefix web ci
npm --prefix web run build:check
node web/scripts/smoke-heramind-upgrade.cjs
```

Script cần Google Chrome hoặc Chromium của Playwright (`CHROME_PATH` để chỉ định executable). Script tạo data dir tạm, chọn cổng loopback còn trống, tắt model tích hợp trong quá trình thử, khởi động/dừng server và giữ log/ảnh/kết quả trong thư mục tạm được in ra. Không dùng tài khoản hoặc camera vận hành. Phiên bản mục tiêu của script là 0.9.24.

Build còn cảnh báo kích thước chunk Settings và dữ liệu Browserslist đã cũ. `lint:ci` kiểm tra lỗi; không đồng nghĩa mọi cảnh báo lint đã được xử lý.

## Khi đưa vào vận hành

1. Dừng phiên bản đang vận hành và sao lưu nhất quán toàn bộ thư mục dữ liệu cùng các secret/key liên quan.
2. Thử bản 0.9.24 trên bản sao dữ liệu với cổng riêng. Kiểm tra thiết bị, dashboard, quy tắc, dữ liệu lịch sử và đăng nhập trước khi thay binary đang vận hành.
3. Extension dùng FFI cũ cần build lại. Kiểm tra riêng model, camera/LAN và kết nối IM thực tế của môi trường triển khai.
4. Nếu rollback, khôi phục cả binary cũ và bản dữ liệu trước nâng cấp. Không mở dữ liệu đã được phiên bản mới ghi bằng binary 0.9.11.

Chưa chạy model thật, camera thật, IM có tài khoản thật hoặc đóng gói/ký release trên macOS/Windows. Kiểm tra desktop tại đây là biên dịch trên Linux. Việc tích hợp không tự tạo ra phân quyền API key chi tiết: upstream vẫn coi các quyền khai báo là metadata.

## Lần đồng bộ tiếp theo

Dùng commit NeoMind v0.9.24 ở trên làm baseline nội dung mới và so với nhánh HeraMind đã tích hợp. Không dùng lại baseline v0.9.11. Giữ ngoại lệ cho URL hệ sinh thái, các alias SDK, EventList, analytics và ngôn ngữ/giao diện được liệt kê trong bảng. Chạy lại kiểm tra placeholder tiếng Việt, telemetry và các bước kiểm chứng bên dưới sau khi ghép.

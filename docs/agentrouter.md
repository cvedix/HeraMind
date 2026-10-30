# Kết nối AgentRouter

## Kết nối đang hoạt động qua OpenClaw

Đã kiểm chứng ngày 29/10/2026 với model được cấp cho khóa là `gpt-6.1-sol`:

```text
HeraMind → http://127.0.0.1:19376/v1 → OpenClaw → AgentRouter /v1/responses
```

OpenClaw 2026.10.8 chạy bằng Node 24.22.2 được cài riêng trong
`data/agentrouter-openclaw/`. Đây là client OpenClaw thật. HeraMind dùng
giao thức Chat Completions với gateway; OpenClaw gọi Responses API của
AgentRouter. Đã nhận HTTP 200 và câu trả lời `OK` qua bộ kiểm tra HeraMind,
đồng thời kiểm chứng stream SSE, gọi hàm và gửi kết quả hàm trở lại model.

Backend HeraMind đang dùng:

| Trường | Giá trị |
| --- | --- |
| Tên | AgentRouter · GPT-6.1 Sol qua OpenClaw |
| Giao thức | OpenAI |
| Endpoint | `http://127.0.0.1:19376/v1` |
| Model phía HeraMind | `openclaw/default` |
| Model thực tế phía AgentRouter | `gpt-6.1-sol` |
| API key phía HeraMind | Token gateway cục bộ, khác khóa AgentRouter |

Chạy từ thư mục dự án:

```bash
node scripts/agentrouter-openclaw.mjs start
```

Giữ tiến trình này chạy cùng backend HeraMind. Dừng bằng Ctrl+C trong cửa sổ
chạy gateway. Trong cửa sổ khác, có thể kiểm tra bằng:

```bash
node scripts/agentrouter-openclaw.mjs status
node scripts/agentrouter-openclaw.mjs test
./target/debug/heramind llm test openai_6e8fc8c5
```

Mẫu cấu hình là `deploy/agentrouter-openclaw.config.json`. Khi khởi động,
launcher chép cấu hình vào thư mục dữ liệu để các snapshot của OpenClaw
không ghi vào mã nguồn. Gateway chỉ lắng nghe loopback, yêu cầu token và
không bật công cụ host hoặc heartbeat định kỳ. Các hàm do HeraMind gửi
qua API vẫn được chuyển tiếp theo hợp đồng Chat Completions.

Thông tin xác thực nằm trong `data/agentrouter-openclaw/secrets.json`
(quyền `0600`, thư mục bị Git bỏ qua), gồm `apiKey` cho AgentRouter và
`gatewayToken` ngẫu nhiên cho cổng cục bộ. Có thể thay bằng biến môi trường
`HERAMIND_AGENTROUTER_KEY` và `HERAMIND_OPENCLAW_GATEWAY_TOKEN`. Không dùng
khóa AgentRouter làm token xác thực gateway. Log của launcher che cả hai khóa.

Giới hạn kiểm chứng: chưa xác nhận hỗ trợ ảnh hoặc mọi biến thể gọi công cụ;
luồng text, SSE và một vòng gọi hàm đơn giản đã hoạt động. Các model ngoài
quyền của khóa như `gpt-6-astra`, `deepseek-v4-flash`, `claude-opus-4-8`
đã trả 403 khi thử qua OpenClaw. Cần giữ Model ID đúng quyền của tài khoản.

## Giới hạn truy cập client

Khi kiểm tra trực tiếp từ HeraMind, AgentRouter hiện trả HTTP 401 với
`type: unauthorized_client_error` và thông báo `unauthorized client detected`.
Đây là phản hồi từ nhà cung cấp về client gửi yêu cầu; không đủ để kết luận
API key không hợp lệ. Endpoint tương thích OpenAI/Anthropic và một khóa đã
lưu chưa đảm bảo nhà cung cấp chấp nhận ứng dụng HeraMind.

Các báo cáo trực tiếp trong kho tài liệu AgentRouter ghi nhận cùng lỗi
với client tùy chỉnh, trong khi khóa đó hoạt động ở client được hỗ trợ:
[issue #37](https://github.com/agentrouter-org/docs/issues/37) và
[issue #21](https://github.com/agentrouter-org/docs/issues/21).

Cần yêu cầu AgentRouter xác nhận hỗ trợ HeraMind, cho phép client hoặc
cung cấp endpoint/header chính thức dành cho ứng dụng tùy chỉnh. Việc đổi
Model ID hay nhập model thủ công không xử lý được lớp từ chối client này.
Chức năng tải model và kiểm tra phản hồi của HeraMind vẫn phụ thuộc vào
quyền truy cập do AgentRouter cấp.

## Cấu hình

Trong **Cài đặt → LLM → Thêm backend AI đám mây**, chọn nhà cung cấp
**AgentRouter**, nhập tên và API key của tài khoản.

Khi rời ô API key, HeraMind tải danh sách model từ
`https://agentrouter.org/v1/models`. Có thể dùng nút **Tải danh sách model**
để thử lại. Nhập tên vào ô model để lọc, rồi chọn một model trong danh sách.
HeraMind giữ nguyên Model ID do nhà cung cấp trả về.

Khi chọn model Claude, HeraMind chọn giao thức Anthropic với endpoint
`https://agentrouter.org`; các model khác dùng giao thức tương thích OpenAI
với endpoint `https://agentrouter.org/v1`. Có thể đổi giao thức nếu model
được cấp cho tài khoản yêu cầu giao thức khác. Đây là cấu hình theo
[hướng dẫn AgentRouter](https://agentrouter.org/docs/openclaw.html).

Lưu backend, **Kiểm tra phản hồi**, rồi kích hoạt sau khi kiểm tra thành công.
Trên thẻ backend, nút **Kiểm tra phản hồi** gửi một yêu cầu thật với câu
nhắc trả lời `OK`, rồi hiển thị nội dung model trả về và thời gian phản hồi.
Nội dung rỗng hoặc lỗi sinh câu trả lời không được tính là thành công.
Danh sách model cho biết các ID do nhà cung cấp công bố; không đảm bảo mọi
model đều đang có kênh phục vụ hoặc hỗ trợ ảnh, gọi công cụ hay một độ dài
ngữ cảnh cụ thể. Chọn theo nhu cầu và quyền truy cập của tài khoản.

Nếu nhà cung cấp từ chối tải danh sách (401/403), giới hạn tần suất hoặc
không hỗ trợ API danh sách model, giao diện hiển thị lỗi và vẫn cho nhập
Model ID thủ công từ bảng điều khiển AgentRouter. Không dùng danh sách
model cố định vì quyền truy cập và model có thể thay đổi.

API khám phá model là `POST /api/llm-backends/providers/agentrouter/models`,
yêu cầu đăng nhập và nhận `{ "api_key": "…" }` hoặc
`{ "backend_id": "…" }` để sử dụng khóa đã lưu của một backend AgentRouter.
Việc khám phá không lưu khóa hay tạo backend. Khóa không nằm trong URL;
server chỉ gửi khóa đến endpoint HTTPS cố định của AgentRouter và không
theo chuyển hướng. Khi sửa backend, khóa đã lưu không được trả về trình duyệt.

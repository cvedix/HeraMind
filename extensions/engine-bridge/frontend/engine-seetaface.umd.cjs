(function (global) {
  'use strict';
  const R = global.React, h = R.createElement;
  const apiOrigin = () => global.__TAURI_INTERNALS__ ? 'http://127.0.0.1:9375' : global.location.origin;
  function token() {
    return sessionStorage.getItem('heramind_api_key') || localStorage.getItem('heramind_token') ||
      sessionStorage.getItem('heramind_token_session') || localStorage.getItem('neomind_token') ||
      sessionStorage.getItem('neomind_token_session') || '';
  }
  async function api(path, body, signal) {
    const response = await fetch(apiOrigin() + '/api' + path, {
      method: body === undefined ? 'GET' : 'POST',
      headers: { Authorization: 'Bearer ' + token(), 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body), signal
    });
    if (!response.ok) throw new Error('Yêu cầu thất bại (HTTP ' + response.status + ').');
    const envelope = await response.json();
    if (envelope.success === false) throw new Error('HeraMind không xử lý được yêu cầu.');
    return envelope.data === undefined ? envelope : envelope.data;
  }
  const command = (name, args, signal) => api('/extensions/engine-bridge/command', { command: name, args }, signal);
  const EngineSeetaFaceDemo = R.forwardRef(function (props, _ref) {
    const device = props.deviceId || 'seetaface-demo';
    const [image, setImage] = R.useState(''), [result, setResult] = R.useState(null);
    const [busy, setBusy] = R.useState(false), [error, setError] = R.useState('');
    const [message, setMessage] = R.useState('Đang đọc kết quả từ HeraMind…');
    const [name, setName] = R.useState(''), [enrolled, setEnrolled] = R.useState(null);
    const current = R.useRef(null), cancelled = R.useRef(false), input = R.useRef(null);
    const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
    async function readLatest(signal) {
      return (await command('get_latest', { device_id: device }, signal)).latest;
    }
    R.useEffect(() => {
      cancelled.current = false;
      const controller = new AbortController(); current.current = controller;
      (async () => {
        try {
          const frame = await api('/devices/' + encodeURIComponent(device) + '/current', undefined, controller.signal);
          const value = frame.metrics && frame.metrics.image && frame.metrics.image.value;
          if (typeof value === 'string' && /^data:image\/(jpeg|png|webp);base64,/.test(value)) setImage(value);
          const latest = await readLatest(controller.signal);
          if (!cancelled.current) {
            setResult(latest);
            setMessage(latest ? 'Đã xử lý bằng SeetaFace qua engine.' : 'Bấm Chạy ảnh mẫu để bắt đầu.');
          }
        } catch (err) { if (!cancelled.current && err.name !== 'AbortError') setError(err.message); }
      })();
      return () => { cancelled.current = true; current.current && current.current.abort(); };
    }, [device]);
    async function process(encoded, signal) {
      const previous = await readLatest(signal);
      await api('/devices/' + encodeURIComponent(device) + '/metrics', { metric: 'image', value: encoded }, signal);
      for (let attempt = 0; attempt < 60; attempt++) {
        if (cancelled.current || signal.aborted) return;
        await wait(500);
        const next = await readLatest(signal);
        if (next && (!previous || next.request_id !== previous.request_id)) {
          if (!cancelled.current) { setImage(encoded); setResult(next); setMessage('Đã xử lý bằng SeetaFace qua engine.'); }
          return;
        }
      }
      throw new Error('Chưa nhận được kết quả. Kiểm tra binding image của thiết bị demo.');
    }
    async function run(action) {
      if (busy) return;
      current.current && current.current.abort();
      const controller = new AbortController(); current.current = controller;
      setBusy(true); setError(''); setMessage('SeetaFace đang xử lý…');
      try { await action(controller.signal); }
      catch (err) { if (!cancelled.current && err.name !== 'AbortError') setError(err.message); }
      finally { if (!cancelled.current) setBusy(false); }
    }
    function sample() {
      return run(async signal => {
        const response = await fetch(apiOrigin() + '/api/extensions/engine-bridge/assets/assets/seetaface-example.png',
          { headers: { Authorization: 'Bearer ' + token() }, signal });
        if (!response.ok) throw new Error('Không tải được ảnh mẫu.');
        const data = await new Promise((resolve, reject) => {
          const reader = new FileReader(); reader.onload = () => resolve(reader.result); reader.onerror = reject;
          response.blob().then(blob => reader.readAsDataURL(blob), reject);
        });
        await process(data, signal);
      });
    }
    function choose(event) {
      const file = event.target.files && event.target.files[0]; event.target.value = '';
      if (!file) return;
      if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type) || file.size > 600 * 1024) {
        setError('Chọn ảnh JPEG, PNG hoặc WebP tối đa 600 KB.'); return;
      }
      run(async signal => {
        const encoded = await new Promise((resolve, reject) => {
          const reader = new FileReader(); reader.onload = () => resolve(reader.result); reader.onerror = reject; reader.readAsDataURL(file);
        });
        await process(encoded, signal);
      });
    }
    async function enroll() {
      if (!name.trim() || !image) return;
      await run(async signal => {
        const response = await command('face_register', { name: name.trim(), image_base64: image }, signal);
        setEnrolled(response.data);
        await process(image, signal);
      });
    }
    async function remove() {
      await run(async signal => {
        await command('face_delete', { face_id: enrolled.face_id }, signal);
        setEnrolled(null); await process(image, signal);
      });
    }
    const data = result && result.result, faces = data && Array.isArray(data.faces) ? data.faces : [];
    const button = { padding: '8px 14px', border: '1px solid #94a3b8', borderRadius: 8, cursor: busy ? 'wait' : 'pointer', background: 'transparent', color: 'inherit' };
    const boxes = [];
    for (let i = 0; i < faces.length; i++) {
      const face = faces[i], b = face.bbox || [0, 0, 0, 0];
      boxes.push(h('rect', { key: 'b' + i, x: b[0], y: b[1], width: b[2], height: b[3], fill: 'none', stroke: '#34d399', strokeWidth: 3 }));
      for (let j = 0; j < (face.keypoints || []).length; j++) boxes.push(h('circle', {
        key: 'p' + i + '-' + j, cx: face.keypoints[j][0], cy: face.keypoints[j][1], r: 3, fill: '#fbbf24'
      }));
    }
    return h('div', { className: 'engine-seetaface-demo', style: { padding: 16, height: '100%', overflow: 'auto', display: 'flex', flexDirection: 'column', gap: 12, color: 'inherit' } },
      h('div', { style: { display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center' } },
        h('strong', null, 'SeetaFace qua Engine'),
        h('span', { style: { fontSize: 12, padding: '3px 8px', borderRadius: 20, background: '#059669', color: '#fff' } }, 'CPU · Model thật')),
      h('p', { style: { margin: 0, fontSize: 13, opacity: .8 } }, 'Ví dụ bằng ảnh mẫu, độc lập với camera USB. Khung xanh là khuôn mặt, chấm vàng là 5 mốc căn chỉnh.'),
      h('div', { style: { display: 'flex', gap: 8, flexWrap: 'wrap' } },
        h('button', { style: button, disabled: busy, onClick: sample }, busy ? 'Đang xử lý…' : 'Chạy ảnh mẫu'),
        h('button', { style: button, disabled: busy, onClick: () => input.current.click() }, 'Chọn ảnh khác'),
        h('input', { ref: input, type: 'file', accept: 'image/jpeg,image/png,image/webp', onChange: choose, style: { display: 'none' } })),
      error ? h('div', { role: 'alert', style: { color: '#ef4444' } }, error) : null,
      h('div', { role: 'status', style: { fontSize: 13 } }, message),
      image ? h('div', { style: { position: 'relative', alignSelf: 'center', width: 'min(100%, 420px)', lineHeight: 0 } },
        h('img', { src: image, alt: 'Ảnh đầu vào SeetaFace', style: { display: 'block', width: '100%', height: 'auto', borderRadius: 8 } }),
        data ? h('svg', { 'aria-label': 'Kết quả khuôn mặt', viewBox: '0 0 ' + data.width + ' ' + data.height,
          style: { position: 'absolute', inset: 0, width: '100%', height: '100%', pointerEvents: 'none' } }, boxes) : null) : null,
      data ? h('div', { style: { display: 'flex', flexWrap: 'wrap', gap: 20, fontSize: 14 } },
        h('strong', { 'data-testid': 'engine-face-count' }, 'Khuôn mặt: ' + faces.length),
        h('span', null, 'Xử lý: ' + Math.round(result.latency_ms) + ' ms'),
        h('span', null, 'Nguồn: ' + device)) : null,
      faces.map((face, i) => h('div', { key: i, style: { fontSize: 14 } },
        h('div', null, 'Danh tính: ' + (face.identity || 'Chưa đăng ký') +
          (face.identity ? ' · Điểm đối chiếu: ' + Number(face.similarity).toFixed(3) : '')),
        data.age_estimation_enabled ? h('div', { 'data-testid': 'engine-face-age', style: { marginTop: 6 } },
          Number.isInteger(face.age_estimate) && face.age_estimation_status === 'estimated'
            ? 'Tuổi ước lượng: khoảng ' + face.age_estimate + ' tuổi'
            : 'Tuổi ước lượng: chưa đủ dữ liệu') : null)),
      data && data.age_estimation_enabled ? h('small', { style: { opacity: .7 } },
        'Tuổi do mô hình ước lượng từ ảnh, có thể sai lệch; không dùng để xác minh tuổi.') : null,
      h('div', { style: { display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center', borderTop: '1px solid #94a3b8', paddingTop: 12 } },
        h('input', { value: name, onChange: e => setName(e.target.value), placeholder: 'Tên muốn đăng ký', 'aria-label': 'Tên đăng ký SeetaFace', maxLength: 128,
          style: { padding: 8, border: '1px solid #94a3b8', borderRadius: 8, color: 'inherit', background: 'transparent' } }),
        h('button', { disabled: busy || faces.length !== 1 || !name.trim() || !!enrolled, style: button, onClick: enroll }, 'Đăng ký ảnh này'),
        enrolled ? h('button', { disabled: busy, style: button, onClick: remove }, 'Xóa đăng ký vừa tạo') : null),
      h('small', { style: { opacity: .7 } }, 'Đăng ký lưu danh tính vào kho SeetaFace của engine. Dùng nút Xóa đăng ký vừa tạo để dọn nhãn thử trước khi đóng trang.')
    );
  });
  global.HeraMindEngineFaces = { EngineSeetaFaceDemo };
})(typeof window !== 'undefined' ? window : globalThis);

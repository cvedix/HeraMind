import { chromium, expect } from '../../web/node_modules/@playwright/test/index.mjs';
import { readFile, writeFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const demo = JSON.parse(await readFile(new URL('.local/seetaface-demo.json', root), 'utf8'));
const key = (await readFile(new URL('.local/usb-face-feed/token', root), 'utf8')).trim();
const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM_PATH || '/home/radxa/.cache/ms-playwright/chromium-1234/chrome-linux/chrome',
  headless: true, args: ['--no-sandbox']
});
let faceId = null;
const errors = [];
let enrollmentCheck = 'passed', deletionCheck = 'passed';
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1200 } });
  page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(key => {
    sessionStorage.setItem('heramind_api_key', key);
    localStorage.setItem('i18nextLng', 'vi');
  }, key);
  await page.goto(demo.dashboard_url);
  const widget = page.locator('.engine-seetaface-demo');
  await expect(widget).toBeVisible({ timeout: 30000 });
  await expect(widget.getByTestId('engine-face-count')).toHaveText('Khuôn mặt: 1', { timeout: 30000 });
  await expect(widget.getByTestId('engine-face-age')).toHaveText(/Tuổi ước lượng: khoảng \d+ tuổi/, { timeout: 30000 });
  await expect(widget.locator('svg rect')).toHaveCount(1);
  await expect(widget.locator('svg circle')).toHaveCount(5);
  await widget.getByRole('button', { name: 'Chạy ảnh mẫu', exact: true }).click();
  await expect(widget.getByRole('button', { name: 'Chạy ảnh mẫu', exact: true })).toBeEnabled({ timeout: 60000 });
  await expect(widget.getByRole('alert')).toHaveCount(0);

  // A user may have already enrolled this sample. Do not create a duplicate
  // identity and assume the matcher will prefer the new equal-score entry.
  // CRUD/matching is always tested separately in a disposable database.
  const sampleIsUnknown = await widget.getByText('Danh tính: Chưa đăng ký', { exact: true }).count() > 0;
  if (sampleIsUnknown) {
  await widget.getByRole('textbox', { name: 'Tên đăng ký SeetaFace' }).fill('Mẫu thử SeetaFace');
  const registered = page.waitForResponse(response => {
    try { return response.url().endsWith('/api/extensions/engine-bridge/command') && response.request().postDataJSON().command === 'face_register'; }
    catch { return false; }
  });
  await widget.getByRole('button', { name: 'Đăng ký ảnh này', exact: true }).click();
  const registration = await (await registered).json();
  faceId = registration.data.data.face_id;
  await expect(widget.getByText(/Danh tính: Mẫu thử SeetaFace/)).toBeVisible({ timeout: 60000 });
  const deleted = page.waitForResponse(response => {
    try { return response.url().endsWith('/api/extensions/engine-bridge/command') && response.request().postDataJSON().command === 'face_delete'; }
    catch { return false; }
  });
  await widget.getByRole('button', { name: 'Xóa đăng ký vừa tạo', exact: true }).click();
  expect((await (await deleted).json()).success).toBe(true);
  faceId = null;
  await expect(widget.getByText('Danh tính: Chưa đăng ký', { exact: true })).toBeVisible({ timeout: 60000 });
  } else {
    enrollmentCheck = 'covered in disposable DB; existing live identity preserved';
    deletionCheck = 'covered in disposable DB; no existing live identity deleted';
  }
  await expect(widget.getByRole('alert')).toHaveCount(0);
  expect(errors).toEqual([]);
  await widget.evaluate(element => { element.scrollTop = 0; });
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: new URL('.local/seetaface-demo.png', root).pathname, fullPage: true });
  const report = { dashboard_url: demo.dashboard_url, faces: 1, landmarks: 5,
    age_estimation: 'displayed as an estimate',
    sample_button: 'passed', enrollment_and_matching: enrollmentCheck, deletion: deletionCheck, browser_errors: errors };
  await writeFile(new URL('.local/seetaface-demo-ui-validation.json', root), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report));
} finally {
  if (faceId !== null) {
    const origin = new URL(demo.dashboard_url).origin;
    await fetch(origin + '/api/extensions/engine-bridge/command', {
      method: 'POST', headers: { Authorization: 'Bearer ' + key, 'Content-Type': 'application/json' },
      body: JSON.stringify({ command: 'face_delete', args: { face_id: faceId } })
    });
  }
  await browser.close();
}

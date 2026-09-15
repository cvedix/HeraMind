// Run after cargo build -p heramind-cli --bin heramind and npm --prefix web run build:check.
// Uses temporary data and loopback ports. Leaves artifacts in the printed scratch directory.
const {createRequire} = require('node:module');
const requireWeb = createRequire(require('node:path').resolve(__dirname, '../package.json'));
const {chromium} = requireWeb('@playwright/test');
const {spawn, execFileSync} = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '../..');
const dataDir = fs.mkdtempSync(path.join(os.tmpdir(), 'heramind-upgrade-smoke-'));
let base, httpPort;
async function freePort() {
 const listener = require('node:net').createServer();
 await new Promise(resolve => listener.listen(0, '127.0.0.1', resolve));
 const port = listener.address().port;
 await new Promise(resolve => listener.close(resolve));
 return port;
}
const serverLog = fs.openSync(path.join(dataDir, 'server.log'), 'w');
let server, browser, token;
const checks = [];
function pass(name) {checks.push(name); console.log('PASS', name);}
async function api(route, method='GET', body, authenticated=true) {
 const res = await fetch(base + '/api' + route, {method, headers:{'Content-Type':'application/json', ...(authenticated && token ? {Authorization:`Bearer ${token}`} : {})}, ...(body ? {body:JSON.stringify(body)} : {})});
 const text = await res.text(); assert.ok(res.ok, `${method} ${route}: ${res.status} ${text.slice(0,500)}`);
 const json = JSON.parse(text); return json.data ?? json;
}
async function start() {
 server = spawn(root+'/target/debug/heramind', ['serve','--host','127.0.0.1','--port',String(httpPort)], {cwd:dataDir, env:{...process.env, HERAMIND_DATA_DIR:dataDir, HERAMIND_WEB_DIR:root+'/web/dist', HERAMIND_BUILTIN_LLM:'off', HERAMIND_MQTT_BIND:'127.0.0.1'}, stdio:['ignore',serverLog,serverLog]});
 for(let i=0; i<120; i++) {
  if(server.exitCode !== null) throw new Error('Server exited '+server.exitCode);
  try { const health=await api('/health'); assert.equal(health.version,'0.9.24'); return; } catch {}
  await new Promise(r=>setTimeout(r,500));
 }
 throw new Error('Startup timeout');
}
async function stop() {
 if(!server || server.exitCode!==null) return;
 const child=server; const exit=new Promise(r=>child.once('exit',r)); child.kill('SIGTERM');
 const killer=setTimeout(()=>child.kill('SIGKILL'),5000); await exit; clearTimeout(killer);
}
(async()=>{
 try {
  httpPort = await freePort();
  let mqttPort = await freePort();
  while (mqttPort === httpPort) mqttPort = await freePort();
  base = `http://127.0.0.1:${httpPort}`;
  fs.writeFileSync(path.join(dataDir, 'config.toml'), `[mqtt]\nlisten = "127.0.0.1"\nport = ${mqttPort}\n`);
  console.log('Scratch data and artifacts:', dataDir);
  await start(); pass('server boots as HeraMind 0.9.24 with isolated data');
  const unauthorized=await fetch(base+'/api/devices'); assert.equal(unauthorized.status,401); pass('device API requires authentication');
  const spec=await api('/docs/openapi.json'); assert.ok(Object.keys(spec.paths).length>250);
  const refs=[]; function visit(v){ if(!v || typeof v!=='object')return; if(v.$ref)refs.push(v.$ref);Object.values(v).forEach(visit); } visit(spec);
  for(const ref of refs.filter(r=>r.startsWith('#/'))) assert.ok(ref.slice(2).split('/').reduce((v,k)=>v?.[k.replace(/~1/g,'/').replace(/~0/g,'~')],spec),ref);
  pass(`OpenAPI: ${Object.keys(spec.paths).length} paths and all local references resolve`);
  browser=await chromium.launch({...(process.env.CHROME_PATH ? {executablePath:process.env.CHROME_PATH} : fs.existsSync('/usr/bin/google-chrome') ? {executablePath:'/usr/bin/google-chrome'} : {}),headless:true,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1440,height:1000},locale:'en-US'});
  const pageErrors=[];page.on('pageerror',err=>pageErrors.push(err.message));
  await page.goto(base); await page.waitForFunction(()=>document.documentElement.lang==='vi');
  await page.screenshot({path:path.join(dataDir, 'setup.png'),fullPage:true});
  pass('fresh browser defaults to Vietnamese even with English browser locale');
  const account=await api('/setup/initialize','POST',{username:'upgrade_smoke',password:'HeraMind-Scratch-2026!'}); token=account.token; assert.ok(token);
  await api('/setup/complete','POST');
  assert.equal((await api('/settings/timezone')).timezone, 'Asia/Ho_Chi_Minh');
  pass('fresh installation uses the Vietnam timezone');
  await page.goto(base+'/login'); await page.locator('#username').fill('upgrade_smoke'); await page.locator('#password').fill('HeraMind-Scratch-2026!'); await page.locator('button[type="submit"]').click();
  await page.waitForURL(url=>!url.pathname.includes('login') && !url.pathname.includes('setup'));
  await page.waitForFunction(()=>Boolean(window.HeraMindStream));
  assert.ok(await page.evaluate(()=>window.NeoMindStream===window.HeraMindStream && window.neomind===window.heramind));
  assert.match(await page.title(),/HeraMind/);
  for(const dark of [false,true]) {
   await page.evaluate(dark=>document.documentElement.classList.toggle('dark',dark),dark);
   const brand=await page.evaluate(()=>getComputedStyle(document.documentElement).getPropertyValue('--brand'));
   assert.match(brand,/255/);
  }
  await page.evaluate(()=>document.documentElement.classList.remove('dark'));
  await page.waitForLoadState('networkidle');
  await page.screenshot({path:path.join(dataDir, 'home.png'),fullPage:true});
  pass('Vietnamese login, HeraMind title, blue light/dark themes and both browser SDK namespaces');
  const type=JSON.parse(fs.readFileSync(root+'/examples/heracam-rv1126b/device-type.json','utf8'));
  await api('/device-types','POST',type);
  await api('/devices','POST',{device_type:type.device_type,device_id:'heracam-upgrade-smoke',name:'HeraCam kiểm thử',adapter_type:'http',connection_config:{simulated:true}});
  const now=Math.floor(Date.now()/1000);
  for(const [i,value] of [4,7,9].entries()) await api('/devices/heracam-upgrade-smoke/metrics','POST',{metric:'vehicle_count',value,timestamp:(now-20+i)*1000});
  const query=`/devices/heracam-upgrade-smoke/telemetry?metric=vehicle_count&start=${now-60}&end=${now+10}`;
  const count=await api(query+'&aggregate=count');
  const sum=await api(query+'&aggregate=sum');
  fs.writeFileSync(path.join(dataDir, 'telemetry.json'),JSON.stringify({count,sum},null,2));
  function value(v) {return v.value ?? v.data?.vehicle_count?.[0]?.value ?? v.data?.[0]?.value ?? v.points?.[0]?.value ?? (v.data ? value(v.data) : undefined);}
  assert.equal(value(count),3);assert.equal(value(sum),20);
  pass('original HeraCam device type, metric ingestion, count=3 and sum=20');
  await api('/devices/heracam-upgrade-smoke/metrics','POST',{metric:'vehicle_count',value:5,timestamp:(now-4000)*1000});
  const history=JSON.parse(execFileSync(root+'/target/debug/heramind', ['device','history','heracam-upgrade-smoke','--metric','vehicle_count','--time-range','1h','--offset','1h','--aggregate','sum','--compress=false'], {cwd:dataDir,env:{...process.env,HERAMIND_DATA_DIR:dataDir,HERAMIND_API_BASE:base+'/api',HERAMIND_API_KEY:token,HERAMIND_JSON:"1"},encoding:'utf8'}));
  fs.writeFileSync(path.join(dataDir, 'cli-history.json'), JSON.stringify(history,null,2));
  assert.equal(value(history),5);
  pass('CLI previous-hour offset and server-side sum remain compatible');
  const dashboard=await api('/dashboards','POST',{name:'HeraMind kiểm thử nâng cấp',components:[{id:'events',type:'event-list',title:'Sự kiện HeraCam',position:{x:0,y:0,w:12,h:6},config:{}}]});
  const dashId=dashboard.id ?? dashboard.dashboard?.id; assert.ok(dashId);
  const loaded=await api('/dashboards/'+dashId); const dash=loaded.dashboard ?? loaded;
  assert.equal(dash.components[0].type,'event-list');
  pass('EventList dashboard saves and reloads');
  for(const route of ['/devices','/visual-dashboard/'+dashId,'/settings']) {
   await page.goto(base+route); await page.waitForLoadState('networkidle');
   assert.ok((await page.locator('body').innerText()).trim().length>50);
   if(route.startsWith('/visual-dashboard/')) await page.getByText('Sự kiện HeraCam', {exact:true}).waitFor();
  }
  await page.screenshot({path:path.join(dataDir, 'settings.png'),fullPage:true});
  assert.deepEqual(pageErrors,[]);pass('devices, dashboard and settings render without browser exceptions');
  await browser.close();browser=null;
  await stop();await start();
  await api('/devices/heracam-upgrade-smoke');await api('/dashboards/'+dashId);
  assert.equal(value(await api(query+'&aggregate=count')),3);
  pass('session, HeraCam telemetry and EventList dashboard survive server restart');
  fs.writeFileSync(path.join(dataDir, 'smoke-results.json'),JSON.stringify({dataDir,checks},null,2));
 } finally {if(browser)await browser.close();await stop();fs.closeSync(serverLog);}
})().catch(err=>{console.error(err);process.exitCode=1;});

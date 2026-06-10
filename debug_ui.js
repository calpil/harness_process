const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

(async () => {
  const targetUrl = process.argv[2] || 'http://localhost:5173';
  let hasErrors = false;
  console.log(`[Playwright] Verificando UI en ${targetUrl}...`);

  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.error(`[Console Error] ${msg.text()}`);
      hasErrors = true;
    }
  });

  page.on('pageerror', exception => {
    console.error(`[Exception] ${exception}`);
    hasErrors = true;
  });

  page.on('requestfailed', request => {
    const failure = request.failure();
    console.error(`[Request Failed] ${request.method()} ${request.url()}${failure ? `: ${failure.errorText}` : ''}`);
    hasErrors = true;
  });

  try {
    await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 20000 });
    await page.waitForLoadState('networkidle', { timeout: 5000 }).catch(() => {});
    await page.waitForSelector('body', { timeout: 5000 });
  } catch (error) {
    console.error(`[Timeout] ${error.message}`);
    hasErrors = true;
  }

  if (hasErrors) {
    const outDir = path.join(__dirname, 'debug-ui');
    fs.mkdirSync(outDir, { recursive: true });
    const shot = path.join(outDir, 'ui_error_state.png');
    try {
      await page.screenshot({ path: shot });
      console.log(`[Playwright] Captura guardada en ${shot}`);
    } catch (error) {
      console.error(`[Playwright] No se pudo guardar captura: ${error.message}`);
    }
    await browser.close();
    process.exit(1);
  }

  await browser.close();
  console.log('[Playwright] Exito: sin errores visibles.');
})();

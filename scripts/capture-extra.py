#!/usr/bin/env python3
"""Capture extra detailed screenshots with test data."""
from playwright.sync_api import sync_playwright
from pathlib import Path
import requests

FRONTEND_URL = "http://localhost:5174"
AUTH = {"username": "Admin", "password": "zxc707cxz"}
SDIR = Path("docs/img")


def get_upload_token():
    resp = requests.post(
        "https://fsx.camthink.ai/api/login",
        json={"username": "auto-upload", "password": "m2;E&@hK$u%nN>1o"},
    )
    return resp.text.strip().strip('"')


def upload(filepath, name, token):
    url = f"https://fsx.camthink.ai/api/resources/heramind-manual/{name}?override=true"
    with open(filepath, "rb") as f:
        resp = requests.post(
            url,
            headers={"X-Auth": token, "Content-Type": "application/octet-stream"},
            data=f,
        )
    return resp.status_code


def snap(page, name, delay=1500):
    page.wait_for_timeout(delay)
    path = str(SDIR / f"{name}.png")
    page.screenshot(path=path)
    print(f"  ✓ {name}")
    return path


def do_login(page):
    page.goto(f"{FRONTEND_URL}/login", wait_until="domcontentloaded", timeout=30000)
    page.wait_for_timeout(3000)
    page.locator("input#username").fill("")
    page.locator("input#username").type(AUTH["username"], delay=50)
    page.locator("input#password").fill("")
    page.locator("input#password").type(AUTH["password"], delay=50)
    page.wait_for_timeout(500)
    btn = page.query_selector('button[type="submit"]')
    if btn:
        try:
            btn.click(timeout=5000)
        except Exception:
            page.evaluate(
                'document.querySelector("button[type=\\"submit\\"]").click()'
            )
        page.wait_for_timeout(5000)
    print(f"  Logged in: {page.url}")


def click_tab(page, text):
    tabs = page.get_by_role("tab")
    for i in range(tabs.count()):
        if text.lower() in tabs.nth(i).inner_text().lower():
            tabs.nth(i).click()
            page.wait_for_timeout(1000)
            return True
    return False


def click_button(page, text):
    btns = page.query_selector_all("button")
    for b in btns:
        if text in b.inner_text() and b.is_visible():
            b.click()
            page.wait_for_timeout(1000)
            return True
    return False


def main():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        ctx = browser.new_context(viewport={"width": 1440, "height": 900}, locale="en-US")
        page = ctx.new_page()

        do_login(page)

        # === DEVICES with data ===
        print("\n=== DEVICES ===")
        page.goto(f"{FRONTEND_URL}/devices", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "devices-with-data")

        # Device types tab (via URL)
        page.goto(f"{FRONTEND_URL}/devices?tab=types", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "device-types-list")

        # Back to devices, click first device row for detail
        page.goto(f"{FRONTEND_URL}/devices", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        rows = page.query_selector_all("tbody tr")
        if rows:
            rows[0].click()
            page.wait_for_timeout(3000)
            snap(page, "device-detail-telemetry")
            page.keyboard.press("Escape")
            page.wait_for_timeout(500)

        # === SETTINGS ===
        print("\n=== SETTINGS ===")
        page.goto(f"{FRONTEND_URL}/settings", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "settings-general-page")

        # LLM tab (via URL)
        page.goto(f"{FRONTEND_URL}/settings?tab=llm", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "settings-llm-page")

        # Retention tab
        page.goto(f"{FRONTEND_URL}/settings?tab=retention", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "settings-retention-page")

        # === AUTOMATION ===
        print("\n=== AUTOMATION ===")
        page.goto(f"{FRONTEND_URL}/automation", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "automation-rules")

        # Transforms tab (via URL)
        page.goto(f"{FRONTEND_URL}/automation?tab=transforms", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "automation-transforms")

        # Data Push tab (via URL)
        page.goto(f"{FRONTEND_URL}/automation?tab=data-push", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "automation-data-push")

        # Data Explorer tab (via URL)
        page.goto(f"{FRONTEND_URL}/automation?tab=data-explorer", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "automation-data-explorer")

        # === AGENTS ===
        print("\n=== AGENTS ===")
        page.goto(f"{FRONTEND_URL}/agents", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "agents-list-empty")

        # Try to open create agent dialog
        if click_button(page, "Create AI Agent"):
            page.wait_for_timeout(2500)
            snap(page, "agent-create-dialog")
            page.keyboard.press("Escape")
            page.wait_for_timeout(500)

        # === MESSAGES ===
        print("\n=== MESSAGES ===")
        page.goto(f"{FRONTEND_URL}/messages", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "messages-channels")

        # === EXTENSIONS ===
        print("\n=== EXTENSIONS ===")
        page.goto(f"{FRONTEND_URL}/extensions", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "extensions-installed")

        # === DASHBOARD ===
        print("\n=== DASHBOARD ===")
        page.evaluate("() => localStorage.setItem('heramind_dashboard_sidebar_open', 'false')")
        page.goto(f"{FRONTEND_URL}/visual-dashboard", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(3000)
        snap(page, "dashboard-main")

        # === CHAT ===
        print("\n=== CHAT ===")
        page.goto(f"{FRONTEND_URL}/", wait_until="domcontentloaded", timeout=30000)
        page.wait_for_timeout(2000)
        snap(page, "chat-main")

        ctx.close()
        browser.close()

    # Upload all new screenshots
    print("\n=== UPLOADING ===")
    token = get_upload_token()
    for f in sorted(SDIR.glob("*.png")):
        code = upload(str(f), f.name, token)
        if code == 200:
            print(f"  ✓ {f.name}")
        elif code == 403:
            print(f"  skip {f.name}")
        else:
            print(f"  ✗ {f.name}: {code}")


if __name__ == "__main__":
    main()

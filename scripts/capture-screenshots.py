#!/usr/bin/env python3
"""
HeraMind Screenshot Capture Script (Extended)
Captures detailed step-by-step screenshots of every page, dialog, and form state.
Uploads to fsx.camthink.ai

Usage:
    python3 scripts/capture-screenshots.py [--skip-upload] [--sections <section,...>]

Sections: login, dashboard, chat, devices, device-dialogs, automation, agents, settings, extensions, messages, mobile, dark

Prerequisites:
    - HeraMind server running on localhost:9375
    - Vite dev server on localhost:5174
    - playwright: pip install playwright && playwright install chromium
    - requests: pip install requests
"""

import argparse
import os
import sys
import time
from pathlib import Path

import requests
from playwright.sync_api import sync_playwright

# Config
FRONTEND_URL = "http://localhost:5174"
API_URL = "http://localhost:9375"
SCRIPT_DIR = Path(__file__).parent
SCREENSHOT_DIR = SCRIPT_DIR.parent / "docs" / "img"
UPLOAD_BASE = "https://fsx.camthink.ai/api"
UPLOAD_PATH = "/resources/heramind-manual"

AUTH = {"username": "Admin", "password": "zxc707cxz"}
UPLOAD_AUTH = {"username": "auto-upload", "password": "m2;E&@hK$u%nN>1o"}


def get_upload_token():
    """Get JWT token for file upload."""
    resp = requests.post(
        f"{UPLOAD_BASE}/login",
        json=UPLOAD_AUTH,
        headers={"Content-Type": "application/json"},
    )
    resp.raise_for_status()
    text = resp.text.strip().strip('"')
    return text


def upload_file(filepath, filename, token):
    """Upload a file to fsx.camthink.ai."""
    url = f"{UPLOAD_BASE}{UPLOAD_PATH}/{filename}?override=true"
    with open(filepath, "rb") as f:
        resp = requests.post(
            url,
            headers={"X-Auth": token, "Content-Type": "application/octet-stream"},
            data=f,
        )
    if resp.status_code == 403:
        del_resp = requests.delete(
            f"{UPLOAD_BASE}{UPLOAD_PATH}/{filename}",
            headers={"X-Auth": token},
        )
        if del_resp.status_code in (200, 204, 404):
            with open(filepath, "rb") as f:
                resp = requests.post(
                    f"{UPLOAD_BASE}{UPLOAD_PATH}/{filename}",
                    headers={"X-Auth": token, "Content-Type": "application/octet-stream"},
                    data=f,
                )
    resp.raise_for_status()
    return True


class ScreenshotCapture:
    def __init__(self, page, screenshot_dir):
        self.page = page
        self.sdir = screenshot_dir
        self.captured = []

    def snap(self, name, delay=1000):
        """Capture a screenshot with a given name."""
        path = str(self.sdir / f"{name}.png")
        self.page.wait_for_timeout(delay)
        self.page.screenshot(path=path)
        print(f"  ✓ {name}.png")
        self.captured.append(name)
        return path

    def goto(self, path, delay=2000):
        """Navigate to a page path."""
        self.page.goto(f"{FRONTEND_URL}{path}", wait_until="domcontentloaded", timeout=30000)
        self.page.wait_for_timeout(delay)

    def click_text(self, text, delay=500):
        """Click element containing text."""
        el = self.page.get_by_text(text, exact=True).first
        if el.is_visible():
            el.click()
            self.page.wait_for_timeout(delay)
            return True
        # Try partial match
        el = self.page.get_by_text(text).first
        if el.is_visible():
            el.click()
            self.page.wait_for_timeout(delay)
            return True
        return False

    def click_role(self, role, name=None, delay=500):
        """Click element by ARIA role."""
        if name:
            el = self.page.get_by_role(role, name=name)
        else:
            el = self.page.get_by_role(role).first
        el.click()
        self.page.wait_for_timeout(delay)

    def click_button(self, text, delay=500):
        """Click a button by text."""
        btn = self.page.get_by_role("button", name=text)
        if btn.count() > 0:
            btn.first.click()
            self.page.wait_for_timeout(delay)
            return True
        # Fallback: find button containing text
        btns = self.page.query_selector_all("button")
        for b in btns:
            if text in b.inner_text():
                b.click()
                self.page.wait_for_timeout(delay)
                return True
        return False

    def click_tab(self, text, delay=800):
        """Click a tab by name."""
        tabs = self.page.get_by_role("tab", name=text)
        if tabs.count() > 0:
            tabs.first.click()
            self.page.wait_for_timeout(delay)
            return True
        return False

    def find_and_click(self, selector, delay=500):
        """Click by CSS selector."""
        el = self.page.query_selector(selector)
        if el:
            el.click()
            self.page.wait_for_timeout(delay)
            return True
        return False

    def close_dialog(self, delay=500):
        """Close any open dialog."""
        # Try clicking overlay or pressing Escape
        self.page.keyboard.press("Escape")
        self.page.wait_for_timeout(delay)

    def fill_field(self, selector, value, delay=300):
        """Fill a form field."""
        el = self.page.query_selector(selector)
        if el:
            el.fill(value)
            self.page.wait_for_timeout(delay)
            return True
        return False


def do_login(page):
    """Login to HeraMind."""
    print("Logging in...")
    page.goto(f"{FRONTEND_URL}/login", wait_until="domcontentloaded", timeout=30000)
    page.wait_for_timeout(3000)

    try:
        page.wait_for_selector("input#username", timeout=5000)
    except Exception:
        if "/login" not in page.url:
            print(f"  Already logged in (at {page.url})")
            return True
        return False

    # Fill credentials using type() to trigger React onChange
    page.locator("input#username").click()
    page.locator("input#username").fill("")
    page.locator("input#username").type(AUTH["username"], delay=50)
    page.wait_for_timeout(300)
    page.locator("input#password").click()
    page.locator("input#password").fill("")
    page.locator("input#password").type(AUTH["password"], delay=50)
    page.wait_for_timeout(500)

    # Try clicking submit - use force or JS click if disabled
    submit = page.query_selector('button[type="submit"]')
    if submit:
        try:
            submit.click(timeout=5000)
        except Exception:
            # Button might be disabled by validation, force click via JS
            page.evaluate("document.querySelector('button[type=\\\"submit\\\"]').click()")
        page.wait_for_timeout(5000)
        print(f"  Logged in. URL: {page.url}")
        return True
    return False


def capture_all(s, sections):
    """Capture all screenshots based on requested sections."""

    # =========================================================================
    # LOGIN
    # =========================================================================
    if "login" in sections:
        print("\n=== LOGIN PAGE ===")
        # Capture login on a fresh page first, then do real login after
        s.page.goto(f"{FRONTEND_URL}/login", wait_until="domcontentloaded", timeout=30000)
        s.page.wait_for_timeout(3000)

        # Empty login form
        s.snap("login")

        # Filled form snapshot
        username_input = s.page.query_selector("input#username")
        password_input = s.page.query_selector("input#password")
        if username_input and password_input:
            username_input.fill("Admin")
            s.page.wait_for_timeout(200)
            password_input.fill("MyPass123")
            s.page.wait_for_timeout(500)
            s.snap("login-filled")
            # Clear for actual login
            username_input.fill("")
            password_input.fill("")

        # Now actually login
        if not do_login(s.page):
            print("Login failed!")
            return
    else:
        # Just login without capturing login page
        if not do_login(s.page):
            print("Login failed!")
            return

    # =========================================================================
    # DASHBOARD
    # =========================================================================
    if "dashboard" in sections:
        print("\n=== DASHBOARD ===")
        s.page.evaluate("() => localStorage.setItem('heramind_dashboard_sidebar_open', 'false')")
        s.goto("/visual-dashboard", 3000)
        s.snap("dashboard_light")

        # Try to open dashboard sidebar
        try:
            sidebar_toggle = s.page.query_selector('[data-sidebar-toggle], button[aria-label*="sidebar"], button[aria-label*="Sidebar"]')
            if not sidebar_toggle:
                # Try finding a hamburger or menu button
                btns = s.page.query_selector_all("button")
                for b in btns:
                    svg = b.query_selector("svg")
                    if svg and b.is_visible():
                        cls = svg.get_attribute("class") or ""
                        # Skip for now
                        pass
        except Exception:
            pass

    # =========================================================================
    # CHAT
    # =========================================================================
    if "chat" in sections:
        print("\n=== CHAT ===")
        s.goto("/", 3000)
        s.snap("chat-empty")

        # Type a message but don't send
        try:
            input_area = s.page.query_selector("textarea, [contenteditable='true'], input[type='text']")
            if input_area:
                input_area.fill("Hello, can you help me monitor my devices?")
                s.page.wait_for_timeout(500)
                s.snap("chat-typing")
                input_area.fill("")
        except Exception as e:
            print(f"  Note: Could not type message: {e}")

        # Check if there are existing sessions
        s.snap("chat-sessions")

        # Try clicking a session if exists
        try:
            session_items = s.page.query_selector_all('[class*="session"], [class*="Session"]')
            if len(session_items) > 1:
                session_items[0].click()
                s.page.wait_for_timeout(2000)
                s.snap("chat-with-history")
        except Exception:
            pass

    # =========================================================================
    # DEVICES - Main page
    # =========================================================================
    if "devices" in sections:
        print("\n=== DEVICES ===")
        s.goto("/devices", 3000)
        s.snap("devices-list")

        # Device types tab
        if s.click_tab("Device Types"):
            s.snap("device-types-tab")
            s.click_tab("Devices")  # go back

        # Drafts tab
        if s.click_tab("Drafts"):
            s.snap("device-drafts-tab")
            s.click_tab("Devices")

    # =========================================================================
    # DEVICES - Dialogs and forms
    # =========================================================================
    if "device-dialogs" in sections:
        print("\n=== DEVICE DIALOGS ===")
        s.goto("/devices", 3000)

        # Add Device dialog
        try:
            if s.click_button("Add Device", 1000) or s.click_button("+", 1000):
                s.page.wait_for_timeout(1500)
                s.snap("device-add-dialog")

                # Try to interact with device type selector
                try:
                    type_select = s.page.query_selector("select, [role='combobox'], button[role='combobox']")
                    if type_select:
                        type_select.click()
                        s.page.wait_for_timeout(800)
                        s.snap("device-add-type-dropdown")
                        s.page.keyboard.press("Escape")
                        s.page.wait_for_timeout(300)
                except Exception:
                    pass

                # Fill in some fields
                try:
                    name_input = s.page.query_selector("input[name='name'], input[placeholder*='name' i], input[id*='name']")
                    if name_input:
                        name_input.fill("Living Room Sensor")
                        s.page.wait_for_timeout(300)
                        s.snap("device-add-filled")
                except Exception:
                    pass

                # Switch adapter to Webhook
                try:
                    adapter_select = s.page.query_selector("[aria-label*='adapter' i], [aria-label*='Adapter']")
                    if not adapter_select:
                        # Find by label text
                        labels = s.page.query_selector_all("label")
                        for lbl in labels:
                            if "adapter" in lbl.inner_text().lower():
                                adapter_select = lbl.query_selector("+ select, + div")
                                break
                    if adapter_select:
                        adapter_select.click()
                        s.page.wait_for_timeout(800)
                        s.snap("device-add-webhook")
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Add device dialog: {e}")

        # Click on first device to see detail
        try:
            s.page.wait_for_timeout(1000)
            rows = s.page.query_selector_all("tr[class*='row'], tbody tr, [class*='device-card'], [class*='DeviceCard']")
            if rows:
                rows[0].click()
                s.page.wait_for_timeout(2000)
                s.snap("device-detail")

                # Look for command button
                try:
                    if s.click_button("Send Command", 1000):
                        s.snap("device-command-dialog")
                        s.close_dialog(500)
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Device detail: {e}")

    # =========================================================================
    # AUTOMATION
    # =========================================================================
    if "automation" in sections:
        print("\n=== AUTOMATION ===")

        # Rules tab
        s.goto("/automation", 3000)
        s.snap("rules-list")

        # Try to open Create Rule dialog
        try:
            if s.click_button("Create Rule", 1000) or s.click_button("Add Rule", 1000) or s.click_button("+ Add", 1000):
                s.page.wait_for_timeout(1500)
                s.snap("rule-create-dialog")

                # Try interacting with condition builder
                try:
                    # Look for device selector
                    selects = s.page.query_selector_all("select, [role='combobox']")
                    for sel in selects:
                        sel.click()
                        s.page.wait_for_timeout(800)
                        s.snap("rule-condition-builder")
                        s.page.keyboard.press("Escape")
                        break
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Rule dialog: {e}")

        # Transforms tab
        try:
            if s.click_tab("Transforms", 1000):
                s.snap("transforms-list")

                # Create transform dialog
                try:
                    if s.click_button("Create Transform", 1000) or s.click_button("Add Transform", 1000) or s.click_button("+ Add", 1000):
                        s.page.wait_for_timeout(1500)
                        s.snap("transform-create-dialog")
                        s.close_dialog(500)
                except Exception as e:
                    print(f"  Note: Transform dialog: {e}")
        except Exception:
            pass

        # Data Explorer / Push tab
        try:
            if s.click_tab("Data Explorer", 1000) or s.click_tab("Explorer", 1000):
                s.snap("data-explorer")

            if s.click_tab("Data Push", 1000) or s.click_tab("Push", 1000):
                s.snap("data-push-list")

                try:
                    if s.click_button("Create Push Target", 1000) or s.click_button("Add Push", 1000) or s.click_button("+ Add", 1000):
                        s.page.wait_for_timeout(1500)
                        s.snap("data-push-create-dialog")
                        s.close_dialog(500)
                except Exception as e:
                    print(f"  Note: Push target dialog: {e}")
        except Exception:
            pass

    # =========================================================================
    # AGENTS
    # =========================================================================
    if "agents" in sections:
        print("\n=== AGENTS ===")
        s.goto("/agents", 3000)
        s.snap("agents-list")

        # Try to open Create Agent dialog
        try:
            if s.click_button("Create Agent", 1000) or s.click_button("Add Agent", 1000) or s.click_button("+ Add", 1000):
                s.page.wait_for_timeout(2000)
                s.snap("agent-create-dialog")

                # Try to switch execution mode
                try:
                    mode_select = s.page.query_selector("[aria-label*='mode' i], [aria-label*='Mode' i], [aria-label*='execution' i]")
                    if mode_select:
                        mode_select.click()
                        s.page.wait_for_timeout(800)
                        s.snap("agent-execution-modes")
                        s.page.keyboard.press("Escape")
                except Exception:
                    pass

                # Try schedule section
                try:
                    schedule_select = s.page.query_selector("[aria-label*='schedule' i], [aria-label*='Schedule' i]")
                    if schedule_select:
                        schedule_select.click()
                        s.page.wait_for_timeout(800)
                        s.snap("agent-schedule-options")
                        s.page.keyboard.press("Escape")
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Agent dialog: {e}")

        # Click on existing agent to see detail
        try:
            s.page.wait_for_timeout(1000)
            cards = s.page.query_selector_all("[class*='agent-card'], [class*='AgentCard'], [class*='card']")
            if cards:
                cards[0].click()
                s.page.wait_for_timeout(2000)
                s.snap("agent-detail")

                # Execution history
                try:
                    if s.click_tab("Executions", 1000) or s.click_tab("History", 1000):
                        s.snap("agent-executions")
                except Exception:
                    pass

                # Memory panel
                try:
                    if s.click_tab("Memory", 1000):
                        s.snap("agent-memory")
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Agent detail: {e}")

    # =========================================================================
    # SETTINGS
    # =========================================================================
    if "settings" in sections:
        print("\n=== SETTINGS ===")
        s.goto("/settings", 3000)
        s.snap("settings-general")

        # LLM Backends tab
        try:
            if s.click_tab("LLM Backends", 1000):
                s.snap("settings-llm-list")

                # Add LLM backend dialog
                try:
                    if s.click_button("Add Backend", 1000) or s.click_button("+ Add", 1000):
                        s.page.wait_for_timeout(1500)
                        s.snap("llm-add-dialog")

                        # Provider dropdown
                        try:
                            provider_select = s.page.query_selector("[aria-label*='provider' i], [aria-label*='Provider' i], select")
                            if provider_select:
                                provider_select.click()
                                s.page.wait_for_timeout(800)
                                s.snap("llm-provider-dropdown")
                                s.page.keyboard.press("Escape")
                        except Exception:
                            pass

                        s.close_dialog(500)
                except Exception as e:
                    print(f"  Note: LLM add dialog: {e}")
        except Exception:
            pass

        # Data retention tab
        try:
            if s.click_tab("Data Retention", 1000) or s.click_tab("Retention", 1000):
                s.snap("settings-retention")
        except Exception:
            pass

    # =========================================================================
    # EXTENSIONS
    # =========================================================================
    if "extensions" in sections:
        print("\n=== EXTENSIONS ===")
        s.goto("/extensions", 3000)
        s.snap("extensions-list")

        # Installed vs Marketplace tabs
        try:
            if s.click_tab("Marketplace", 1000) or s.click_tab("Available", 1000):
                s.snap("extensions-marketplace")
                s.click_tab("Installed")
        except Exception:
            pass

        # Click on extension detail
        try:
            cards = s.page.query_selector_all("[class*='extension-card'], [class*='ExtensionCard'], [class*='card']")
            if cards:
                cards[0].click()
                s.page.wait_for_timeout(1500)
                s.snap("extension-detail")
                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Extension detail: {e}")

    # =========================================================================
    # MESSAGES / NOTIFICATIONS
    # =========================================================================
    if "messages" in sections:
        print("\n=== MESSAGES / NOTIFICATIONS ===")
        s.goto("/messages", 3000)
        s.snap("messages-list")

        # Try to create a notification channel
        try:
            if s.click_button("Create Channel", 1000) or s.click_button("Add Channel", 1000) or s.click_button("+ Add", 1000):
                s.page.wait_for_timeout(1500)
                s.snap("channel-create-dialog")

                # Try channel type dropdown
                try:
                    type_select = s.page.query_selector("[aria-label*='type' i], [aria-label*='Type' i], select")
                    if type_select:
                        type_select.click()
                        s.page.wait_for_timeout(800)
                        s.snap("channel-type-dropdown")
                        s.page.keyboard.press("Escape")
                except Exception:
                    pass

                s.close_dialog(500)
        except Exception as e:
            print(f"  Note: Channel dialog: {e}")

        # Messages tab (delivery log)
        try:
            if s.click_tab("Messages", 1000) or s.click_tab("Delivery Log", 1000):
                s.snap("messages-delivery-log")
        except Exception:
            pass

    # =========================================================================
    # DARK MODE
    # =========================================================================
    if "dark" in sections:
        print("\n=== DARK MODE ===")
        s.page.evaluate("() => { localStorage.setItem('theme', 'dark'); }")
        s.page.evaluate("() => document.documentElement.classList.add('dark')")
        s.page.evaluate("() => localStorage.setItem('heramind_dashboard_sidebar_open', 'false')")

        s.goto("/visual-dashboard", 3000)
        s.page.evaluate("() => document.documentElement.classList.add('dark')")
        s.page.wait_for_timeout(2000)
        s.snap("dashboard_dark")

        # Reset to light
        s.page.evaluate("() => { localStorage.setItem('theme', 'light'); document.documentElement.classList.remove('dark'); }")

    # =========================================================================
    # MOBILE
    # =========================================================================
    if "mobile" in sections:
        print("\n=== MOBILE (separate context) ===")
        # This section needs a separate browser context and is handled in main()


def capture_mobile(browser, screenshot_dir):
    """Capture mobile screenshots in a separate browser context."""
    print("\n=== MOBILE ===")
    mobile_ctx = browser.new_context(
        viewport={"width": 390, "height": 844},
        locale="en-US",
        is_mobile=True,
        has_touch=True,
        user_agent="Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15",
    )
    mobile_page = mobile_ctx.new_page()
    s = ScreenshotCapture(mobile_page, screenshot_dir)

    try:
        do_login(mobile_page)
    except Exception as e:
        print(f"  Mobile login failed: {e}")

    mobile_page.evaluate("() => localStorage.setItem('heramind_dashboard_sidebar_open', 'false')")

    # Dashboard
    try:
        s.goto("/visual-dashboard", 3000)
        s.snap("mobile_dashboard")
    except Exception as e:
        print(f"  Mobile dashboard: {e}")

    # Chat
    try:
        s.goto("/", 3000)
        s.snap("mobile_chat")
    except Exception as e:
        print(f"  Mobile chat: {e}")

    # Devices
    try:
        s.goto("/devices", 3000)
        s.snap("mobile_devices")
    except Exception as e:
        print(f"  Mobile devices: {e}")

    # Mobile web overview
    try:
        s.goto("/", 2000)
        s.snap("mobile_web")
    except Exception as e:
        print(f"  Mobile overview: {e}")

    mobile_ctx.close()


def main():
    all_sections = ["login", "dashboard", "chat", "devices", "device-dialogs", "automation",
                    "agents", "settings", "extensions", "messages", "dark", "mobile"]

    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-upload", action="store_true")
    parser.add_argument("--sections", default=",".join(all_sections),
                        help=f"Comma-separated sections. Options: {', '.join(all_sections)}")
    args = parser.parse_args()

    sections = [s.strip() for s in args.sections.split(",")]
    SCREENSHOT_DIR.mkdir(parents=True, exist_ok=True)

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)

        # Desktop context
        context = browser.new_context(
            viewport={"width": 1440, "height": 900}, locale="en-US"
        )
        page = context.new_page()

        # Pre-set sidebar closed
        page.goto(f"{FRONTEND_URL}/login", wait_until="domcontentloaded", timeout=30000)
        page.evaluate("() => localStorage.setItem('heramind_dashboard_sidebar_open', 'false')")

        s = ScreenshotCapture(page, SCREENSHOT_DIR)

        # Capture main sections
        capture_all(s, [sec for sec in sections if sec != "mobile"])

        context.close()

        # Mobile in separate context
        if "mobile" in sections:
            capture_mobile(browser, SCREENSHOT_DIR)

        browser.close()

    print(f"\nCaptured {len(list(SCREENSHOT_DIR.glob('*.png')))} screenshots total")
    print(f"New captures: {len(s.captured)}")

    # Upload
    if args.skip_upload:
        print("\nSkipping upload (--skip-upload)")
        return

    print("\n=== UPLOADING ===")
    try:
        token = get_upload_token()
        print("  Got upload token")
    except Exception as e:
        print(f"  Failed to get token: {e}")
        return

    uploaded = 0
    for f in sorted(SCREENSHOT_DIR.glob("*.png")):
        try:
            upload_file(str(f), f.name, token)
            print(f"  ✓ {f.name}")
            uploaded += 1
        except Exception as e:
            print(f"  ✗ {f.name}: {e}")

    print(f"\nUploaded {uploaded} screenshots to https://fsx.camthink.ai{UPLOAD_PATH}/")


if __name__ == "__main__":
    main()

const DEFAULT_API_URL = "http://localhost:3000/api";

const statusBar = document.getElementById("statusBar");
const titleInput = document.getElementById("titleInput");
const urlPreview = document.getElementById("urlPreview");
const contentPreview = document.getElementById("contentPreview");
const clipBtn = document.getElementById("clipBtn");
const projectSelect = document.getElementById("projectSelect");
const settingsBtn = document.getElementById("settingsBtn");
const settingsPanel = document.getElementById("settingsPanel");
const apiInput = document.getElementById("apiInput");
const tokenInput = document.getElementById("tokenInput");
const saveSettingsBtn = document.getElementById("saveSettingsBtn");
const cancelSettingsBtn = document.getElementById("cancelSettingsBtn");

let extractedContent = "";
let pageUrl = "";
let projects = [];

async function getSettings() {
  const result = await chrome.storage.local.get(["apiUrl", "accessToken"]);
  return {
    apiUrl: result.apiUrl || DEFAULT_API_URL,
    accessToken: result.accessToken || "",
  };
}

async function saveSettings(settings) {
  await chrome.storage.local.set(settings);
}

async function checkConnection() {
  const settings = await getSettings();

  if (!settings.accessToken) {
    statusBar.className = "status disconnected";
    statusBar.textContent = "✗ Not authenticated";
    clipBtn.disabled = true;
    projectSelect.innerHTML = '<option value="">Login required</option>';
    showSettings();
    return false;
  }

  try {
    const res = await fetch(`${settings.apiUrl}/health`, {
      method: "GET",
      headers: { "Authorization": `Bearer ${settings.accessToken}` },
    });

    if (res.ok) {
      statusBar.className = "status connected";
      statusBar.textContent = "✓ Connected to LLM Wiki";
      await loadProjects(settings);
      return true;
    }
  } catch {}

  statusBar.className = "status disconnected";
  statusBar.textContent = "✗ Cannot connect to server";
  clipBtn.disabled = true;
  projectSelect.innerHTML = '<option value="">Connection failed</option>';
  return false;
}

async function loadProjects(settings) {
  try {
    const res = await fetch(`${settings.apiUrl}/projects/list`, {
      method: "GET",
      headers: { "Authorization": `Bearer ${settings.accessToken}` },
    });

    if (res.ok) {
      projects = await res.json();
      projectSelect.innerHTML = "";

      if (projects.length === 0) {
        projectSelect.innerHTML = '<option value="">No projects</option>';
        return;
      }

      for (const proj of projects) {
        const opt = document.createElement("option");
        opt.value = proj.id;
        opt.textContent = proj.name;
        projectSelect.appendChild(opt);
      }
      return;
    }
  } catch {}

  projectSelect.innerHTML = '<option value="">Failed to load</option>';
}

async function extractContent() {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id) return;

    pageUrl = tab.url || "";
    titleInput.value = tab.title || "Untitled";
    urlPreview.textContent = pageUrl;

    await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      files: ["Readability.js", "Turndown.js"],
    });

    const results = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => {
        try {
          const documentClone = document.cloneNode(true);
          const reader = new window.Readability(documentClone);
          const article = reader.parse();

          if (!article || !article.content) {
            return { error: "Readability could not extract content" };
          }

          const turndown = new window.TurndownService({
            headingStyle: "atx",
            codeBlockStyle: "fenced",
            bulletListMarker: "-",
          });

          turndown.addRule("tableCell", {
            filter: ["th", "td"],
            replacement: (content) => ` ${content.trim()} |`,
          });
          turndown.addRule("tableRow", {
            filter: "tr",
            replacement: (content) => `|${content}\n`,
          });
          turndown.addRule("table", {
            filter: "table",
            replacement: (content) => {
              const lines = content.trim().split("\n");
              if (lines.length > 0) {
                const cols = (lines[0].match(/\|/g) || []).length - 1;
                const separator = "|" + " --- |".repeat(cols);
                lines.splice(1, 0, separator);
              }
              return "\n\n" + lines.join("\n") + "\n\n";
            },
          });

          turndown.addRule("removeSmallImages", {
            filter: (node) => {
              if (node.nodeName !== "IMG") return false;
              const w = parseInt(node.getAttribute("width") || "999");
              const h = parseInt(node.getAttribute("height") || "999");
              return w < 10 || h < 10;
            },
            replacement: () => "",
          });

          const markdown = turndown.turndown(article.content);

          return {
            title: article.title,
            content: markdown,
            excerpt: article.excerpt || "",
            siteName: article.siteName || "",
            length: article.length || 0,
          };
        } catch (err) {
          return { error: err.message };
        }
      },
    });

    if (results?.[0]?.result) {
      const result = results[0].result;

      if (result.error) {
        contentPreview.textContent = `Extraction failed: ${result.error}. Falling back...`;
        await fallbackExtract(tab.id);
        return;
      }

      if (result.title && result.title.length > 5) {
        titleInput.value = result.title;
      }

      extractedContent = result.content;
      contentPreview.textContent = extractedContent;

      if (result.excerpt) {
        contentPreview.textContent = "📝 " + result.excerpt + "\n\n---\n\n" + extractedContent;
      }

      clipBtn.disabled = false;
    } else {
      await fallbackExtract(tab.id);
    }
  } catch (err) {
    contentPreview.textContent = `Error: ${err.message}`;
  }
}

async function fallbackExtract(tabId) {
  const results = await chrome.scripting.executeScript({
    target: { tabId },
    func: () => {
      const clone = document.body.cloneNode(true);
      ["script", "style", "nav", "header", "footer", ".sidebar", ".ad", ".comments"]
        .forEach((sel) => clone.querySelectorAll(sel).forEach((el) => el.remove()));

      return clone.innerText
        .split("\n")
        .map((l) => l.trim())
        .filter((l) => l.length > 0)
        .join("\n\n")
        .slice(0, 50000);
    },
  });

  if (results?.[0]?.result) {
    extractedContent = results[0].result;
    contentPreview.textContent = extractedContent;
    clipBtn.disabled = false;
  } else {
    contentPreview.textContent = "Failed to extract content";
  }
}

async function sendClip() {
  const selectedProject = projectSelect.value;
  if (!selectedProject) {
    statusBar.className = "status error";
    statusBar.textContent = "✗ Please select a project";
    return;
  }

  const settings = await getSettings();
  if (!settings.accessToken) {
    statusBar.className = "status error";
    statusBar.textContent = "✗ Not authenticated";
    showSettings();
    return;
  }

  clipBtn.disabled = true;
  statusBar.className = "status sending";
  statusBar.textContent = "⏳ Sending to LLM Wiki...";

  try {
    const res = await fetch(`${settings.apiUrl}/ingest/clip`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Authorization": `Bearer ${settings.accessToken}`,
      },
      body: JSON.stringify({
        project_id: selectedProject,
        title: titleInput.value,
        url: pageUrl,
        content: extractedContent,
      }),
    });

    const data = await res.json();

    if (res.ok) {
      const projectName = projectSelect.options[projectSelect.selectedIndex]?.textContent || "project";
      statusBar.className = "status success";
      statusBar.textContent = `✓ Saved to ${projectName}`;
      clipBtn.textContent = "✓ Clipped!";
      setTimeout(() => {
        clipBtn.textContent = "📎 Clip to Wiki";
        clipBtn.disabled = false;
      }, 2000);
    } else {
      statusBar.className = "status error";
      statusBar.textContent = `✗ Error: ${data.error || "Unknown error"}`;
      clipBtn.disabled = false;
    }
  } catch (err) {
    statusBar.className = "status error";
    statusBar.textContent = `✗ Connection failed: ${err.message}`;
    clipBtn.disabled = false;
  }
}

function showSettings() {
  settingsPanel.style.display = "block";
}

function hideSettings() {
  settingsPanel.style.display = "none";
}

async function handleSaveSettings() {
  const apiUrl = apiInput.value.trim() || DEFAULT_API_URL;
  const accessToken = tokenInput.value.trim();

  if (!accessToken) {
    alert("Please enter your access token");
    return;
  }

  await saveSettings({ apiUrl, accessToken });
  hideSettings();
  await checkConnection();
}

clipBtn.addEventListener("click", sendClip);
settingsBtn.addEventListener("click", showSettings);
saveSettingsBtn.addEventListener("click", handleSaveSettings);
cancelSettingsBtn.addEventListener("click", hideSettings);

function resizePreview() {
  const totalHeight = 500;
  const preview = document.getElementById("contentPreview");
  if (!preview) return;

  const previewRect = preview.getBoundingClientRect();
  const bottomSpace = totalHeight - previewRect.top - 60;
  const maxH = Math.max(100, Math.min(300, bottomSpace));
  preview.style.maxHeight = maxH + "px";
}

(async () => {
  const settings = await getSettings();
  apiInput.value = settings.apiUrl === DEFAULT_API_URL ? "" : settings.apiUrl;
  tokenInput.value = settings.accessToken ? "••••••••" : "";

  const connected = await checkConnection();
  await extractContent();

  if (!connected) {
    clipBtn.disabled = true;
    clipBtn.textContent = "📎 Login required — cannot save";
  }

  setTimeout(resizePreview, 100);
})();

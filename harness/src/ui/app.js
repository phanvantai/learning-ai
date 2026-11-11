"use strict";

// The access token arrives once in the URL fragment and is kept for this tab only.
const token = (() => {
  const fromHash = new URLSearchParams(location.hash.slice(1)).get("token");
  if (fromHash) {
    sessionStorage.setItem("lai-token", fromHash);
    history.replaceState(null, "", location.pathname);
  }
  return sessionStorage.getItem("lai-token") || "";
})();

let state = null;
let view = "chat";
let chat = { sessionId: null, busy: false, messages: [] };

// ---------------------------------------------------------------- helpers ---

async function api(method, path, body) {
  const res = await fetch("/api" + path, {
    method,
    headers: { "x-lai-token": token, ...(body ? { "content-type": "application/json" } : {}) },
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || `Lỗi ${res.status}`);
  return data;
}

/** h("div", {class: "x", onclick: fn}, child, "text") */
function h(tag, attrs, ...children) {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs || {})) {
    if (value == null || value === false) continue;
    if (key.startsWith("on")) el.addEventListener(key.slice(2), value);
    else if (key === "class") el.className = value;
    else if (key === "value") el.value = value;
    else el.setAttribute(key, value === true ? "" : value);
  }
  for (const child of children.flat(Infinity)) {
    if (child == null || child === false) continue;
    el.append(child.nodeType ? child : document.createTextNode(String(child)));
  }
  return el;
}

const $ = (id) => document.getElementById(id);

let toastTimer;
function toast(message, isError) {
  const el = $("toast");
  el.textContent = message;
  el.className = isError ? "error" : "";
  el.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (el.hidden = true), isError ? 7000 : 3000);
}

/** Run an action from a button: disable it meanwhile, show failures, refresh after. */
async function act(button, fn, doneMessage) {
  const label = button.textContent;
  button.disabled = true;
  try {
    await fn();
    if (doneMessage) toast(doneMessage);
  } catch (e) {
    toast(e.message, true);
  } finally {
    button.disabled = false;
    button.textContent = label;
    await refresh();
  }
}

function escapeHtml(text) {
  return text.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
}

/** Small Markdown subset. Input is escaped first, so the output is safe to insert. */
function markdown(source) {
  const inline = (text) =>
    text
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/(^|[\s(])\*([^*\s][^*]*)\*/g, "$1<em>$2</em>")
      .replace(/\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
  const out = [];
  let list = null;
  let para = [];
  const flush = () => {
    if (para.length) out.push(`<p>${inline(para.join("<br>"))}</p>`);
    para = [];
    if (list) out.push(`</${list}>`);
    list = null;
  };
  const lines = escapeHtml(source).split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.startsWith("```")) {
      flush();
      const code = [];
      for (i++; i < lines.length && !lines[i].startsWith("```"); i++) code.push(lines[i]);
      out.push(`<pre><code>${code.join("\n")}</code></pre>`);
      continue;
    }
    const heading = line.match(/^#{1,4}\s+(.*)/);
    const item = line.match(/^\s*(?:([-*•])|(\d+)[.)])\s+(.*)/);
    if (heading) {
      flush();
      out.push(`<h3>${inline(heading[1])}</h3>`);
    } else if (item) {
      const kind = item[1] ? "ul" : "ol";
      if (list !== kind) {
        flush();
        out.push(`<${kind}>`);
        list = kind;
      }
      out.push(`<li>${inline(item[3])}</li>`);
    } else if (!line.trim()) {
      flush();
    } else {
      if (list) flush();
      para.push(line);
    }
  }
  flush();
  return out.join("");
}

function rich(className, text) {
  const el = h("div", { class: className });
  el.innerHTML = markdown(text);
  return el;
}

function slugify(text) {
  return text
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[đĐ]/g, "d")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

const WEEKDAYS = ["Chủ nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"];
const pad = (n) => String(n).padStart(2, "0");

/** Cron expression -> one of the simple shapes the form offers, or "custom". */
function parseSchedule(cron) {
  const [minute, hour, day, month, weekday] = cron.trim().split(/\s+/);
  const isNumber = (v) => /^\d+$/.test(v);
  if (day !== "*" || month !== "*" || !isNumber(minute)) return { kind: "custom" };
  if (hour === "*" && weekday === "*") return { kind: "hourly", minute: +minute };
  if (!isNumber(hour)) return { kind: "custom" };
  const time = `${pad(hour)}:${pad(minute)}`;
  if (weekday === "*") return { kind: "daily", time };
  if (weekday === "1-5") return { kind: "weekdays", time };
  if (/^[0-7]$/.test(weekday)) return { kind: "weekly", time, weekday: +weekday % 7 };
  return { kind: "custom" };
}

function describeSchedule(cron) {
  const s = parseSchedule(cron);
  if (s.kind === "hourly") return `Mỗi giờ, vào phút ${pad(s.minute)}`;
  if (s.kind === "daily") return `Hằng ngày lúc ${s.time}`;
  if (s.kind === "weekdays") return `Thứ Hai đến thứ Sáu lúc ${s.time}`;
  if (s.kind === "weekly") return `${WEEKDAYS[s.weekday]} hằng tuần lúc ${s.time}`;
  return `Lịch tùy chỉnh: ${cron}`;
}

function buildSchedule(kind, time, weekday, custom) {
  const [hour, minute] = (time || "07:00").split(":").map(Number);
  if (kind === "hourly") return `${minute} * * * *`;
  if (kind === "daily") return `${minute} ${hour} * * *`;
  if (kind === "weekdays") return `${minute} ${hour} * * 1-5`;
  if (kind === "weekly") return `${minute} ${hour} * * ${weekday}`;
  return custom.trim();
}

function reportLabel(file) {
  const m = file.match(/^(\d{4})-(\d{2})-(\d{2})_(\d{2})(\d{2})/);
  return m ? `${m[3]}/${m[2]} ${m[4]}:${m[5]}` : file;
}

function openDialog(...content) {
  $("dialog-body").replaceChildren(...content);
  if (!$("dialog").open) $("dialog").showModal();
}
const closeDialog = () => $("dialog").close();

// ------------------------------------------------------------------ shell ---

async function refresh() {
  try {
    state = await api("GET", "/state");
  } catch (e) {
    toast(e.message, true);
    return;
  }
  const pending = state.pending.length;
  $("pending-badge").textContent = pending;
  $("pending-badge").hidden = pending === 0;
  $("setup-dot").hidden = setupProblems().length === 0;
  $("version").textContent = `Phiên bản ${state.version}`;
  render();
}

function currentAgent() {
  return state.agents.find((a) => a.name === state.default_agent) || state.agents[0];
}

/** What still stops the default agent from working, in plain words. */
function setupProblems() {
  const problems = [];
  const agent = currentAgent();
  if (!agent) return ["Chưa có agent nào trong cấu hình."];
  const provider = state.providers.find((p) => p.name === agent.provider);
  if (provider && !provider.ready) problems.push(`Bộ não "${provider.name}" chưa sẵn sàng: ${provider.status}.`);
  if (agent.tools.some((t) => t.startsWith("gmail_")) && !state.gmail.connected) problems.push("Chưa kết nối Gmail.");
  return problems;
}

function show(next) {
  view = next;
  for (const item of document.querySelectorAll(".nav-item")) item.classList.toggle("active", item.dataset.view === view);
  for (const section of document.querySelectorAll(".view")) section.hidden = section.id !== `view-${view}`;
  render();
}

function render() {
  if (!state) return;
  if (view === "chat") renderChat();
  if (view === "jobs") renderJobs();
  if (view === "approvals") renderApprovals();
  // Re-rendering settings would wipe what the user is typing; only do it when asked.
  if (view === "settings" && !$("settings").dataset.rendered) renderSettings();
}

// ------------------------------------------------------------------- chat ---

const STARTERS = [
  "Hôm nay có thư nào cần tôi trả lời không?",
  "Tóm tắt các thư chưa đọc trong 24 giờ qua.",
  "Tìm thư gần nhất về hóa đơn và cho tôi biết số tiền, hạn thanh toán.",
];

function renderChat() {
  const select = $("chat-agent");
  select.hidden = state.agents.length < 2;
  if (!select.options.length) {
    for (const agent of state.agents) select.append(h("option", { value: agent.name }, agent.name));
    select.value = state.default_agent || "";
  }

  const box = $("transcript");
  const nodes = [];
  if (!chat.messages.length) {
    const problems = setupProblems();
    nodes.push(
      h(
        "div",
        { class: "empty" },
        problems.length
          ? [
              h("p", null, "Còn bước thiết lập chưa xong:"),
              problems.map((p) => h("p", null, p)),
              h("button", { class: "btn primary", onclick: () => show("settings") }, "Mở mục Kết nối"),
            ]
          : [
              h("p", null, "Giao việc cho trợ lý bằng lời bình thường. Ví dụ:"),
              h("div", { class: "starters" }, STARTERS.map((text) => h("button", { class: "btn", onclick: () => sendChat(text) }, text))),
            ],
      ),
    );
  }
  for (const m of chat.messages) {
    if (m.role === "user") nodes.push(h("div", { class: "msg user" }, m.text));
    else if (m.role === "error") nodes.push(h("div", { class: "msg error" }, m.text));
    else nodes.push(rich("msg bot", m.text));
  }
  if (chat.busy) nodes.push(h("div", { class: "msg note thinking" }, "Trợ lý đang làm việc"));
  if (!chat.busy && chat.messages.length && state.pending.length) {
    nodes.push(
      h(
        "div",
        { class: "msg note" },
        `Có ${state.pending.length} hành động đang chờ bạn duyệt. `,
        h("button", { class: "btn small", onclick: () => show("approvals") }, "Xem"),
      ),
    );
  }
  box.replaceChildren(...nodes);
  box.scrollTop = box.scrollHeight;
  $("chat-send").disabled = chat.busy;
}

async function sendChat(text) {
  text = text.trim();
  if (!text || chat.busy) return;
  chat.messages.push({ role: "user", text });
  chat.busy = true;
  renderChat();
  try {
    const reply = await api("POST", "/chat", {
      session_id: chat.sessionId,
      agent: $("chat-agent").value || null,
      message: text,
    });
    chat.sessionId = reply.session_id;
    chat.messages.push({ role: "bot", text: reply.answer || "(Trợ lý không trả lời gì.)" });
  } catch (e) {
    chat.messages.push({ role: "error", text: e.message });
  }
  chat.busy = false;
  await refresh();
}

function newChat() {
  if (chat.sessionId) api("DELETE", `/chat/${chat.sessionId}`).catch(() => {});
  chat = { sessionId: null, busy: false, messages: [] };
  renderChat();
}

// ------------------------------------------------------------------- jobs ---

function renderJobs() {
  const cards = state.jobs.map((job) => {
    const runButton = h("button", { class: "btn small", disabled: job.running }, job.running ? "Đang chạy…" : "Chạy ngay");
    runButton.addEventListener("click", () => {
      runButton.textContent = "Đang chạy…";
      act(runButton, async () => {
        const result = await api("POST", `/jobs/${job.name}/run`);
        showReport(job.name, result.file, result.report);
      });
    });
    const deleteButton = h("button", { class: "btn small danger" }, "Xóa");
    deleteButton.addEventListener("click", () => {
      if (confirm(`Xóa việc "${job.name}"? Các báo cáo cũ vẫn được giữ trên máy.`)) {
        act(deleteButton, () => api("DELETE", `/jobs/${job.name}`), "Đã xóa.");
      }
    });
    return h(
      "div",
      { class: "card" },
      h(
        "div",
        { class: "card-head" },
        h("div", null, h("h2", null, job.name), h("div", { class: "meta" }, describeSchedule(job.schedule), job.next_run ? ` · lần tới: ${job.next_run}` : "")),
        h("div", { class: "row" }, runButton, h("button", { class: "btn small", onclick: () => editJob(job) }, "Sửa"), deleteButton),
      ),
      h("p", { class: "meta" }, job.prompt.trim().slice(0, 220) + (job.prompt.trim().length > 220 ? "…" : "")),
      job.reports.length
        ? h(
            "div",
            { class: "reports" },
            h("span", { class: "meta" }, "Báo cáo:"),
            job.reports.slice(0, 8).map((file) => h("button", { class: "btn small ghost", onclick: () => openReport(job.name, file) }, reportLabel(file))),
          )
        : h("div", { class: "meta" }, "Chưa có báo cáo nào."),
    );
  });
  $("jobs").replaceChildren(...(cards.length ? cards : [h("div", { class: "empty" }, "Chưa có việc nào. Bấm “Thêm việc” để trợ lý tự làm một việc theo giờ bạn chọn.")]));
}

async function openReport(job, file) {
  try {
    const data = await api("GET", `/reports/${job}/${file}`);
    showReport(job, file, data.text);
  } catch (e) {
    toast(e.message, true);
  }
}

function showReport(job, file, text) {
  openDialog(
    h("h2", null, job),
    h("div", { class: "meta" }, file ? reportLabel(file) : ""),
    rich("report", text),
    h("div", { class: "actions" }, h("button", { class: "btn", onclick: closeDialog }, "Đóng")),
  );
}

function editJob(job) {
  const isNew = !job;
  const parsed = job ? parseSchedule(job.schedule) : { kind: "weekdays", time: "07:00" };
  const title = h("input", { value: job ? job.name : "", placeholder: "Ví dụ: Tóm tắt mail sáng", disabled: !isNew });
  const kind = h(
    "select",
    null,
    [
      ["daily", "Hằng ngày"],
      ["weekdays", "Thứ Hai đến thứ Sáu"],
      ["weekly", "Hằng tuần"],
      ["hourly", "Mỗi giờ"],
      ["custom", "Tùy chỉnh (cron)"],
    ].map(([value, label]) => h("option", { value }, label)),
  );
  kind.value = parsed.kind;
  const weekday = h("select", null, [1, 2, 3, 4, 5, 6, 0].map((d) => h("option", { value: d }, WEEKDAYS[d])));
  weekday.value = parsed.weekday ?? 1;
  const time = h("input", { type: "time", value: parsed.time || `00:${pad(parsed.minute || 0)}` });
  const custom = h("input", { value: job ? job.schedule : "", placeholder: "phút giờ ngày tháng thứ, ví dụ: 30 8 1 * *" });
  const prompt = h("textarea", { rows: 6, placeholder: "Mô tả việc trợ lý cần làm và báo cáo cần có gì." });
  prompt.value = job ? job.prompt.trim() : "";
  const timeHint = h("span", { class: "hint" });
  const sync = () => {
    weekday.hidden = kind.value !== "weekly";
    time.hidden = kind.value === "custom";
    custom.hidden = kind.value !== "custom";
    timeHint.textContent = kind.value === "hourly" ? "Chỉ dùng số phút của giờ đã chọn." : "";
  };
  kind.addEventListener("change", sync);
  sync();

  const save = h("button", { class: "btn primary" }, "Lưu");
  save.addEventListener("click", () => {
    const name = isNew ? slugify(title.value) : job.name;
    if (!name) return toast("Hãy đặt tên cho việc này.", true);
    if (isNew && state.jobs.some((j) => j.name === name)) return toast(`Đã có việc tên "${name}".`, true);
    act(save, async () => {
      await api("PUT", `/jobs/${name}`, {
        agent: job ? job.agent : null,
        schedule: buildSchedule(kind.value, time.value, weekday.value, custom.value),
        prompt: prompt.value,
      });
      closeDialog();
    }, "Đã lưu.");
  });
  openDialog(
    h("h2", null, isNew ? "Thêm việc theo lịch" : `Sửa việc: ${job.name}`),
    h("label", null, "Tên"),
    title,
    h("label", null, "Khi nào"),
    h("div", { class: "row" }, kind, weekday, h("div", null, time), h("div", { class: "grow" }, custom)),
    timeHint,
    h("label", null, "Việc cần làm"),
    prompt,
    h("p", { class: "hint" }, "Khi chạy theo lịch, trợ lý không tự gửi thư: mọi hành động gửi đi sẽ nằm trong mục Chờ duyệt."),
    h("div", { class: "actions" }, h("button", { class: "btn", onclick: closeDialog }, "Hủy"), save),
  );
}

// -------------------------------------------------------------- approvals ---

function renderApprovals() {
  const cards = state.pending.map((action) => {
    const approve = h("button", { class: "btn primary" }, action.tool === "gmail_send" ? "Đồng ý gửi" : "Đồng ý");
    approve.addEventListener("click", () => act(approve, () => api("POST", `/approvals/${action.id}/approve`), "Đã thực hiện."));
    const reject = h("button", { class: "btn" }, "Từ chối");
    reject.addEventListener("click", () => act(reject, () => api("POST", `/approvals/${action.id}/reject`), "Đã từ chối."));
    const input = action.input || {};
    const body =
      action.tool === "gmail_send"
        ? [
            h("dl", { class: "mail-field" }, h("dt", null, "Gửi tới"), h("dd", null, input.to || ""), h("dt", null, "Tiêu đề"), h("dd", null, input.subject || "")),
            h("div", { class: "mail-body" }, input.body || ""),
          ]
        : [h("pre", { class: "raw" }, JSON.stringify(input, null, 2))];
    return h(
      "div",
      { class: "card" },
      h(
        "div",
        { class: "card-head" },
        h("div", null, h("h2", null, action.tool === "gmail_send" ? "Gửi thư" : action.tool), h("div", { class: "meta" }, `Agent ${action.agent} · ${action.created_at.slice(0, 16).replace("T", " ")}`)),
      ),
      body,
      h("div", { class: "actions" }, reject, approve),
    );
  });
  $("approvals").replaceChildren(...(cards.length ? cards : [h("div", { class: "empty" }, "Không có gì chờ duyệt.")]));
}

// --------------------------------------------------------------- settings ---

const KIND_LABEL = {
  "claude-cli": "Gói Claude (qua Claude Code)",
  "codex-cli": "Gói ChatGPT (qua Codex CLI, thử nghiệm)",
  anthropic: "API key Anthropic",
  openai: "API key tương thích OpenAI",
};
const KIND_HELP = {
  "claude-cli": "Cài Claude Code rồi chạy `claude` và đăng nhập một lần trên máy này.",
  "codex-cli": "Cài Codex CLI rồi chạy `codex login` một lần trên máy này.",
};

function renderSettings() {
  const root = $("settings");
  root.dataset.rendered = "1";
  const agent = currentAgent();
  const rerender = async () => {
    delete root.dataset.rendered;
    await refresh();
  };
  const sections = [];

  // 1. Brain
  if (agent) {
    const pick = h("select", null, state.providers.map((p) => h("option", { value: p.name }, `${p.name} — ${KIND_LABEL[p.kind]}`)));
    pick.value = agent.provider;
    pick.addEventListener("change", () =>
      act(pick, () => api("PUT", `/agents/${agent.name}`, { provider: pick.value, instructions: agent.instructions }), "Đã đổi bộ não.").then(rerender),
    );
    sections.push(
      h(
        "div",
        { class: "card" },
        h("h2", null, "Bộ não đang dùng"),
        h("p", { class: "hint" }, "Trợ lý suy nghĩ bằng mô hình AI nào. Có thể dùng gói subscription đã đăng nhập trên máy, hoặc API key."),
        pick,
      ),
    );
  }

  // 2. Providers
  const providerCards = state.providers.map((p) => {
    const rows = [
      h(
        "div",
        { class: "card-head" },
        h("div", null, h("h3", null, p.name), h("div", { class: "meta" }, KIND_LABEL[p.kind], p.model ? ` · ${p.model}` : "")),
        h("div", { class: "status" }, h("span", { class: `dot ${p.ready ? "ok" : "warn"}` }), p.ready ? "Sẵn sàng" : "Chưa sẵn sàng"),
      ),
      h("div", { class: "meta" }, p.status, !p.ready && KIND_HELP[p.kind] ? `. ${KIND_HELP[p.kind]}` : ""),
    ];
    if (p.kind === "anthropic" || p.kind === "openai") {
      const key = h("input", { type: "password", placeholder: "Dán API key mới", autocomplete: "off" });
      const save = h("button", { class: "btn small" }, "Lưu key");
      save.addEventListener("click", () => act(save, () => api("POST", `/providers/${p.name}/key`, { key: key.value }), "Đã lưu key.").then(rerender));
      rows.push(h("div", { class: "row" }, h("div", { class: "grow" }, key), save));
    }
    const remove = h("button", { class: "btn small danger" }, "Gỡ");
    remove.addEventListener("click", () => {
      if (confirm(`Gỡ bộ não "${p.name}"?`)) act(remove, () => api("DELETE", `/providers/${p.name}`), "Đã gỡ.").then(rerender);
    });
    rows.push(h("div", { class: "actions" }, remove));
    return h("div", { class: "card" }, rows);
  });
  sections.push(
    h("div", { class: "view-head" }, h("h2", null, "Các bộ não đã khai báo"), h("button", { class: "btn small", onclick: () => addProvider(rerender) }, "Thêm bộ não")),
    ...providerCards,
  );

  // 3. Gmail
  const gmailRows = [
    h(
      "div",
      { class: "card-head" },
      h("h2", null, "Gmail"),
      h("div", { class: "status" }, h("span", { class: `dot ${state.gmail.connected ? "ok" : "warn"}` }), state.gmail.connected ? "Đã kết nối" : "Chưa kết nối"),
    ),
    h("p", { class: "hint" }, "Trợ lý được quyền đọc thư và gửi thư. Việc gửi luôn phải qua bước bạn duyệt."),
  ];
  const clientId = h("input", { placeholder: "Client ID", autocomplete: "off" });
  const clientSecret = h("input", { type: "password", placeholder: "Client secret", autocomplete: "off" });
  if (!state.gmail.builtin_client) {
    gmailRows.push(
      h("p", { class: "hint" }, "Cần OAuth client loại “Desktop app” của Google Cloud (xem hướng dẫn trong README). Để trống nếu đã nhập trước đó."),
      h("div", { class: "row" }, h("div", { class: "grow" }, clientId), h("div", { class: "grow" }, clientSecret)),
    );
  }
  const connect = h("button", { class: "btn primary" }, state.gmail.connected ? "Kết nối lại" : "Kết nối Gmail");
  connect.addEventListener("click", () => {
    connect.textContent = "Đang chờ bạn đăng nhập trên trình duyệt…";
    act(connect, () => api("POST", "/gmail/login", { client_id: clientId.value, client_secret: clientSecret.value }), "Đã kết nối Gmail.").then(rerender);
  });
  gmailRows.push(h("div", { class: "actions" }, connect));
  sections.push(h("div", { class: "card" }, gmailRows));

  // 4. Instructions
  if (agent) {
    const instructions = h("textarea", { rows: 5, placeholder: "Ví dụ: Tôi làm kế toán. Ưu tiên thư từ khách hàng và ngân hàng. Xưng hô em - anh/chị khi soạn thư." });
    instructions.value = agent.instructions.trim();
    const save = h("button", { class: "btn" }, "Lưu");
    save.addEventListener("click", () => act(save, () => api("PUT", `/agents/${agent.name}`, { provider: agent.provider, instructions: instructions.value }), "Đã lưu.").then(rerender));
    sections.push(
      h(
        "div",
        { class: "card" },
        h("h2", null, "Dặn dò riêng cho trợ lý"),
        h("p", { class: "hint" }, "Những điều trợ lý cần luôn nhớ về bạn và cách bạn muốn làm việc."),
        instructions,
        h("div", { class: "actions" }, save),
      ),
    );
  }

  // 5. Start with the computer
  if (state.autostart !== null) {
    const toggle = h("button", { class: "btn" }, state.autostart ? "Tắt" : "Bật");
    toggle.addEventListener("click", () => act(toggle, () => api("POST", "/autostart", { enabled: !state.autostart })).then(rerender));
    sections.push(
      h(
        "div",
        { class: "card" },
        h(
          "div",
          { class: "card-head" },
          h("div", null, h("h2", null, "Tự mở cùng máy tính"), h("p", { class: "hint" }, `Để việc theo lịch chạy đúng giờ mà không cần nhớ mở app. Hiện đang ${state.autostart ? "bật" : "tắt"}.`)),
          toggle,
        ),
      ),
    );
  }

  sections.push(h("p", { class: "hint" }, `File cấu hình: ${state.config_path}`));
  root.replaceChildren(...sections);
}

function addProvider(done) {
  const kind = h("select", null, Object.entries(KIND_LABEL).map(([value, label]) => h("option", { value }, label)));
  const name = h("input", { placeholder: "Tên gợi nhớ, ví dụ: openai" });
  const model = h("input", { placeholder: "Tên model" });
  const baseUrl = h("input", { placeholder: "https://api.openai.com/v1" });
  const key = h("input", { type: "password", placeholder: "API key", autocomplete: "off" });
  const modelRow = h("div", null, h("label", null, "Model"), model, h("p", { class: "hint", id: "model-hint" }));
  const urlRow = h("div", null, h("label", null, "Địa chỉ API"), baseUrl, h("p", { class: "hint" }, "Để trống nếu dùng OpenAI. Điền địa chỉ khác cho Gemini, OpenRouter, Ollama…"));
  const keyRow = h("div", null, h("label", null, "API key"), key);
  const sync = () => {
    const isApi = kind.value === "anthropic" || kind.value === "openai";
    urlRow.hidden = kind.value !== "openai";
    keyRow.hidden = !isApi;
    modelRow.querySelector(".hint").textContent =
      kind.value === "openai" ? "Bắt buộc, ghi đúng tên model của nhà cung cấp." : "Có thể để trống để dùng mặc định.";
  };
  kind.addEventListener("change", sync);
  sync();
  const save = h("button", { class: "btn primary" }, "Thêm");
  save.addEventListener("click", () => {
    const id = slugify(name.value);
    if (!id) return toast("Hãy đặt tên cho bộ não.", true);
    if (state.providers.some((p) => p.name === id)) return toast(`Đã có bộ não tên "${id}".`, true);
    const body = { kind: kind.value };
    if (model.value.trim()) body.model = model.value.trim();
    if (kind.value === "openai" && baseUrl.value.trim()) body.base_url = baseUrl.value.trim();
    if (!keyRow.hidden && key.value.trim()) body.api_key = key.value.trim();
    act(save, async () => {
      await api("PUT", `/providers/${id}`, body);
      closeDialog();
    }, "Đã thêm.").then(done);
  });
  openDialog(
    h("h2", null, "Thêm bộ não"),
    h("label", null, "Loại"),
    kind,
    h("label", null, "Tên"),
    name,
    modelRow,
    urlRow,
    keyRow,
    h("div", { class: "actions" }, h("button", { class: "btn", onclick: closeDialog }, "Hủy"), save),
  );
}

// ------------------------------------------------------------------- boot ---

for (const item of document.querySelectorAll(".nav-item")) {
  item.addEventListener("click", () => {
    if (item.dataset.view === "settings") delete $("settings").dataset.rendered;
    show(item.dataset.view);
  });
}
$("composer").addEventListener("submit", (event) => {
  event.preventDefault();
  const input = $("chat-input");
  const text = input.value;
  input.value = "";
  sendChat(text);
});
$("chat-input").addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
    event.preventDefault();
    $("composer").requestSubmit();
  }
});
// Links leave the app: hand them to the system browser instead of the app window.
document.addEventListener("click", (event) => {
  const link = event.target.closest && event.target.closest("a[href^='http']");
  if (!link) return;
  event.preventDefault();
  api("POST", "/open", { url: link.href }).catch((e) => toast(e.message, true));
});
$("chat-new").addEventListener("click", newChat);
$("chat-agent").addEventListener("change", newChat);
$("job-add").addEventListener("click", () => editJob(null));

if (!token) {
  document.querySelector("main").replaceChildren(h("div", { class: "empty" }, "Trang này cần được mở từ app (hoặc lệnh `lai ui`)."));
} else {
  show("chat");
  refresh();
  // Keep the pending badge and job status fresh; skip while a dialog is open.
  setInterval(() => !$("dialog").open && refresh(), 15000);
}

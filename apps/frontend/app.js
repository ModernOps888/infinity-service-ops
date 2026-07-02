const timelineItems = [
  {
    tone: "danger",
    label: "P1 Incident",
    title: "Mail bridge degraded in UK South",
    body: "Monitoring signals were normalized, deduplicated, linked through the CMDB, and opened as a tenant-safe incident.",
  },
  {
    tone: "success",
    label: "Policy evaluation",
    title: "Emergency change requires two-person approval",
    body: "High-risk execution paused until the incident manager and platform security roles approve the workflow.",
  },
  {
    tone: "success",
    label: "SecOps",
    title: "Critical vulnerability case promoted from finding",
    body: "Evidence and remediation tasks were attached with chain-of-custody metadata and rollback requirements.",
  },
];

const automationTemplates = [
  {
    name: "Microsoft 365 major incident bridge",
    description: "Event ingested → incident opened → bridge notification → approval gate.",
    guardrails: ["approval-gate", "audit-log", "webhook-validation"],
  },
  {
    name: "Okta access request fulfillment",
    description: "Catalog request → manager approval → least-privilege check → auditable vendor handoff.",
    guardrails: ["manager-approval", "least-privilege", "audit-log"],
  },
  {
    name: "Critical vulnerability remediation",
    description: "Finding import → security case → patch workflow → closure evidence.",
    guardrails: ["two-person-approval", "rollback-runbook", "maintenance-window", "audit-log"],
  },
];

const cases = [
  {
    id: "SEC-1042",
    title: "Critical CVE on regulated tenant workload",
    body: "Finding promoted into a security case with attached scanner evidence and remediation owner.",
  },
  {
    id: "SEC-1037",
    title: "Suspicious privileged access pattern",
    body: "Identity event sequence correlated with service graph dependency and queued for review.",
  },
  {
    id: "SEC-1029",
    title: "Endpoint baseline drift",
    body: "Intune and Defender signals routed to the control plane with audit coverage.",
  },
];

const ledgerItems = [
  ["10:18", "tenant.boundary.checked", "UK South residency verified"],
  ["10:19", "automation.guardrail.evaluated", "Two-person approval required"],
  ["10:20", "ai.egress.blocked", "External inference disabled for regulated data"],
  ["10:21", "audit.record.appended", "Incident and policy evaluation linked"],
];

const escapeHtml = (value) =>
  String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");

const renderTimeline = () => {
  const timeline = document.querySelector("#timeline");
  timeline.innerHTML = timelineItems
    .map(
      (item) => `
        <article class="timeline-item ${item.tone === "danger" ? "danger" : ""}">
          <span class="timeline-marker" aria-hidden="true"></span>
          <div>
            <p class="eyebrow">${escapeHtml(item.label)}</p>
            <strong>${escapeHtml(item.title)}</strong>
            <p>${escapeHtml(item.body)}</p>
          </div>
        </article>
      `,
    )
    .join("");
};

const renderAutomation = () => {
  const grid = document.querySelector("#automation-grid");
  grid.innerHTML = automationTemplates
    .map(
      (template) => `
        <article class="panel automation-card">
          <p class="eyebrow">Enterprise template</p>
          <strong>${escapeHtml(template.name)}</strong>
          <p>${escapeHtml(template.description)}</p>
          <div class="guardrails">
            ${template.guardrails.map((guardrail) => `<span>${escapeHtml(guardrail)}</span>`).join("")}
          </div>
        </article>
      `,
    )
    .join("");
};

const renderCases = () => {
  const list = document.querySelector("#case-list");
  list.innerHTML = cases
    .map(
      (item) => `
        <article class="case-item">
          <p class="eyebrow">${escapeHtml(item.id)}</p>
          <strong>${escapeHtml(item.title)}</strong>
          <p>${escapeHtml(item.body)}</p>
        </article>
      `,
    )
    .join("");
};

const renderLedger = () => {
  const ledger = document.querySelector("#ledger");
  const count = document.querySelector("#ledger-count");
  count.textContent = `${ledgerItems.length} events`;
  ledger.innerHTML = ledgerItems
    .map(
      ([time, event, description]) => `
        <article class="ledger-item">
          <span>${escapeHtml(time)}</span>
          <strong>${escapeHtml(event)}</strong>
          <small>${escapeHtml(description)}</small>
        </article>
      `,
    )
    .join("");
};

const setActiveView = (viewId) => {
  document.querySelectorAll(".workspace-panel").forEach((panel) => {
    panel.classList.toggle("active", panel.id === viewId);
  });

  document.querySelectorAll(".nav-item").forEach((item) => {
    const active = item.dataset.view === viewId;
    item.classList.toggle("active", active);
    item.setAttribute("aria-current", active ? "page" : "false");
  });
};

document.querySelectorAll(".nav-item").forEach((button) => {
  button.addEventListener("click", () => setActiveView(button.dataset.view));
});

document.querySelector("#simulate-policy").addEventListener("click", () => {
  ledgerItems.unshift(["now", "policy.evaluation.simulated", "Regulated change requires approval and audit obligations"]);
  renderLedger();
  setActiveView("sovereignty");
});

document.querySelector("#route-approval").addEventListener("click", () => {
  ledgerItems.unshift(["now", "workflow.approval.routed", "Incident manager and platform security approval requested"]);
  timelineItems.splice(1, 0, {
    tone: "success",
    label: "Approval routed",
    title: "Change approval request sent to control roles",
    body: "The workflow remains paused until both approvers complete policy obligations.",
  });
  document.querySelector("#incident-count").textContent = "4";
  renderTimeline();
  renderLedger();
  setActiveView("overview");
});

renderTimeline();
renderAutomation();
renderCases();
renderLedger();
setActiveView("overview");

import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import "./App.css";

type AgentId = "chat" | "video" | "image";
type Message = {
  id: string;
  role: "user" | "assistant";
  content: string;
  status?: "thinking" | "error";
};
type StreamEvent = {
  taskId: string;
  agentId: AgentId;
  kind: "started" | "thinking" | "completed" | "error";
  content?: string;
  detail?: string;
};
type RuntimeState = {
  isRunning: boolean;
  hasUnread: boolean;
  flashUntil: number | null;
};
type SettingsInfo = {
  apiKeyConfigured: boolean;
  baseUrl: string;
  model: string;
  imageModel: string;
  videoModel: string;
};

const agents: Array<{
  id: AgentId;
  name: string;
  description: string;
  icon: string;
}> = [
  {
    id: "chat",
    name: "聊天",
    description: "和你的 AI 助手对话，处理日常问题和复杂任务。",
    icon: "/chat-icon.png",
  },
  {
    id: "video",
    name: "视频制作",
    description: "从脚本、素材到生成任务，管理你的视频创作。",
    icon: "/video-icon.png",
  },
  {
    id: "image",
    name: "图片制作",
    description: "生成、编辑和整理你的图片素材。",
    icon: "/image-icon.png",
  },
];
const initialRuntime: Record<AgentId, RuntimeState> = {
  chat: { isRunning: false, hasUnread: false, flashUntil: null },
  video: { isRunning: false, hasUnread: false, flashUntil: null },
  image: { isRunning: false, hasUnread: false, flashUntil: null },
};

// 将 Agent 事件中的结果转换为聊天区内容。
function formatMessage(event: StreamEvent): string {
  return event.content?.trim() || "Agent 没有返回可展示的内容。";
}

function App() {
  const [currentApp, setCurrentApp] = useState<AgentId | "home">("chat");
  const [openApps, setOpenApps] = useState<AgentId[]>(["chat"]);
  const [runtime, setRuntime] = useState(initialRuntime);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settings, setSettings] = useState<SettingsInfo | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [viewTransition, setViewTransition] = useState<
    "opening" | "minimizing" | null
  >(null);
  const [contextMenu, setContextMenu] = useState<{
    agentId: AgentId;
    x: number;
    y: number;
  } | null>(null);
  const currentAppRef = useRef<AgentId | "home">("chat");
  const activeTaskId = useRef<string | null>(null);

  useEffect(() => {
    currentAppRef.current = currentApp;
  }, [currentApp]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    // step.1 订阅所有 Agent 事件，统一更新任务栏运行和提醒状态
    listen<StreamEvent>("agent://stream", (event) => {
      const payload = event.payload;
      if (
        payload.agentId === "chat" &&
        (payload.kind === "completed" || payload.kind === "error") &&
        payload.taskId !== activeTaskId.current
      )
        return;
      setRuntime((current) => ({
        ...current,
        [payload.agentId]: {
          ...current[payload.agentId],
          isRunning: payload.kind === "started" || payload.kind === "thinking",
          hasUnread:
            payload.kind === "completed" || payload.kind === "error"
              ? currentAppRef.current !== payload.agentId
              : current[payload.agentId].hasUnread,
          flashUntil:
            payload.kind === "completed" || payload.kind === "error"
              ? currentAppRef.current === payload.agentId
                ? Date.now() + 1500
                : null
              : current[payload.agentId].flashUntil,
        },
      }));
      if (payload.agentId === "chat" && payload.kind === "thinking")
        setMessages((current) =>
          current.map((message) =>
            message.status === "thinking"
              ? { ...message, content: payload.detail ?? "正在处理请求" }
              : message,
          ),
        );
      if (
        payload.agentId === "chat" &&
        (payload.kind === "completed" || payload.kind === "error")
      )
        setMessages((current) =>
          current.map((message) =>
            message.status === "thinking"
              ? {
                  ...message,
                  content:
                    payload.kind === "error"
                      ? (payload.content ?? "请求失败")
                      : formatMessage(payload),
                  status: payload.kind === "error" ? "error" : undefined,
                }
              : message,
          ),
        );
      if (
        (payload.kind === "completed" || payload.kind === "error") &&
        currentAppRef.current === payload.agentId
      )
        window.setTimeout(
          () =>
            setRuntime((current) => ({
              ...current,
              [payload.agentId]: {
                ...current[payload.agentId],
                flashUntil: null,
              },
            })),
          1500,
        );
    }).then((cleanup) => {
      unlisten = cleanup;
    });
    invoke<SettingsInfo>("get_config")
      .then(setSettings)
      .catch(() => undefined);
    return () => unlisten?.();
  }, []);

  // 打开 Agent App，并清除该 App 已读提醒。
  function openAgent(agentId: AgentId) {
    if (currentApp === agentId) {
      minimizeCurrentApp();
      return;
    }
    setOpenApps((current) =>
      current.includes(agentId) ? current : [...current, agentId],
    );
    setCurrentApp(agentId);
    setViewTransition("opening");
    window.setTimeout(() => setViewTransition(null), 1000);
    setContextMenu(null);
    setRuntime((current) => ({
      ...current,
      [agentId]: { ...current[agentId], hasUnread: false, flashUntil: null },
    }));
  }

  // 将当前 Agent 工作区收回任务栏并回到 Home。
  function minimizeCurrentApp() {
    if (currentApp === "home") return;
    setViewTransition("minimizing");
    window.setTimeout(() => {
      setCurrentApp("home");
      setViewTransition(null);
    }, 1000);
  }

  // 关闭任务栏中的 Agent App，不删除它自己的业务数据。
  function closeAgent(agentId: AgentId) {
    setOpenApps((current) => current.filter((id) => id !== agentId));
    if (currentApp === agentId) setCurrentApp("home");
    setContextMenu(null);
    if (agentId === "chat" && activeTaskId.current) {
      void invoke("cancel_chat_task", { taskId: activeTaskId.current }).catch(
        () => undefined,
      );
      activeTaskId.current = null;
    }
    setRuntime((current) => ({
      ...current,
      [agentId]: { ...current[agentId], isRunning: false },
    }));
  }

  // 发送 Chat Agent 消息，并在任务栏中标记运行状态。
  async function sendChatMessage() {
    const text = draft.trim();
    if (!text || runtime.chat.isRunning) return;
    // step.1 写入用户消息和等待中的助手消息
    const userMessage: Message = {
      id: `user-${Date.now()}`,
      role: "user",
      content: text,
    };
    const pending: Message = {
      id: `pending-${Date.now()}`,
      role: "assistant",
      content: "正在准备请求...",
      status: "thinking",
    };
    const history = [...messages, userMessage];
    setMessages([...history, pending]);
    setDraft("");
    // step.2 调用 Chat Agent 适配接口并记录任务标识
    try {
      activeTaskId.current = await invoke<string>("start_chat_task", {
        request: {
          messages: history.map(({ role, content }) => ({ role, content })),
        },
      });
      setRuntime((current) => ({
        ...current,
        chat: { ...current.chat, isRunning: true },
      }));
    } catch (error) {
      setMessages((current) =>
        current.map((message) =>
          message.status === "thinking"
            ? { ...message, content: String(error), status: "error" }
            : message,
        ),
      );
    }
  }

  // 取消当前 Chat Agent 的等待状态。
  async function cancelChatMessage() {
    if (!activeTaskId.current) return;
    const taskId = activeTaskId.current;
    activeTaskId.current = null;
    setRuntime((current) => ({
      ...current,
      chat: { ...current.chat, isRunning: false },
    }));
    setMessages((current) =>
      current.map((message) =>
        message.status === "thinking"
          ? { ...message, content: "已取消生成", status: "error" }
          : message,
      ),
    );
    await invoke("cancel_chat_task", { taskId }).catch(() => undefined);
  }

  // 保存 API key 并刷新设置摘要。
  async function saveSettings() {
    if (!apiKey.trim() && settings?.apiKeyConfigured) {
      setIsSettingsOpen(false);
      return;
    }
    const result = await invoke<SettingsInfo>("save_config", {
      config: { api_key: apiKey.trim() || null },
    });
    setSettings(result);
    setApiKey("");
    setIsSettingsOpen(false);
  }

  return (
    <div className="desktop-shell" onClick={() => setContextMenu(null)}>
      <main
        className={`workspace ${
          viewTransition === "opening"
            ? "genie-open"
            : viewTransition === "minimizing"
              ? "genie-minimize"
              : ""
        }`}
      >
        <header className="workspace-header">
          {currentApp === "home" ? (
            <div className="workspace-context">
              <span className="workspace-context-title">应用</span>
            </div>
          ) : (
            <button
              className="workspace-app-button"
              onClick={minimizeCurrentApp}
              aria-label={`收起 ${agents.find((agent) => agent.id === currentApp)?.name}`}
              title="收起当前 App"
            >
              <img
                src={agents.find((agent) => agent.id === currentApp)?.icon}
                alt=""
              />
              <span>{agents.find((agent) => agent.id === currentApp)?.name}</span>
            </button>
          )}
        </header>
        {currentApp === "home" ? (
          <Home onOpen={openAgent} />
        ) : currentApp === "chat" ? (
          <ChatApp
            messages={messages}
            draft={draft}
            isRunning={runtime.chat.isRunning}
            onDraftChange={setDraft}
            onSend={() => void sendChatMessage()}
            onCancel={() => void cancelChatMessage()}
          />
        ) : currentApp === "video" ? (
          <VideoApp />
        ) : (
          <ImageApp />
        )}
      </main>
      <footer className="taskbar" onClick={(event) => event.stopPropagation()}>
        <button
          className={`taskbar-home ${currentApp === "home" ? "selected" : ""}`}
          onClick={minimizeCurrentApp}
          aria-label="返回 Home"
        >
          ⌂
        </button>
        <div className="taskbar-divider" />
        <div className="running-apps">
          {openApps.map((agentId) => {
            const agent = agents.find((item) => item.id === agentId)!;
            const state = runtime[agentId];
            const isFlashing =
              state.flashUntil !== null && state.flashUntil > Date.now();
            return (
              <button
                key={agentId}
                className={`taskbar-app ${
                  currentApp === agentId ? "selected" : ""
                } ${state.hasUnread ? "unread" : ""} ${
                  state.isRunning ? "running" : ""
                } ${isFlashing ? "flash" : ""}`}
                onClick={() => openAgent(agentId)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  setContextMenu({
                    agentId,
                    x: event.clientX,
                    y: event.clientY,
                  });
                }}
                aria-label={`打开 ${agent.name}`}
                title={agent.name}
              >
                <img className="taskbar-icon" src={agent.icon} alt="" />
                <span className="taskbar-name">{agent.name}</span>
                {state.hasUnread && <span className="unread-dot" />}
              </button>
            );
          })}
        </div>
        {contextMenu && (
          <div
            className="context-menu"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button onClick={() => closeAgent(contextMenu.agentId)}>
              关闭 App
            </button>
          </div>
        )}
        <span className="taskbar-hint">右键任务栏 App 可关闭</span>
        <button
          className="taskbar-settings"
          onClick={() => setIsSettingsOpen(true)}
          aria-label="打开设置"
          title="设置"
        >
          ⚙<span>设置</span>
        </button>
      </footer>
      {isSettingsOpen && (
        <SettingsModal
          settings={settings}
          apiKey={apiKey}
          showKey={showKey}
          onApiKeyChange={setApiKey}
          onToggleKey={() => setShowKey((value) => !value)}
          onClose={() => setIsSettingsOpen(false)}
          onSave={() => void saveSettings()}
        />
      )}
    </div>
  );
}

// Home 页面展示可启动的 Agent App。
function Home({ onOpen }: { onOpen: (agentId: AgentId) => void }) {
  return (
    <section className="home-page">
      <div className="home-heading">
        <span className="eyebrow">APPLICATIONS</span>
        <h1>选择一个 App 开始</h1>
        <p>每个 Agent 都有自己的工作空间和状态。</p>
      </div>
      <div className="app-grid">
        {agents.map((agent) => (
          <button
            className="app-card"
            key={agent.id}
            onClick={() => onOpen(agent.id)}
          >
            <img className="app-icon" src={agent.icon} alt="" />
            <span className="app-card-title">{agent.name}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

// Chat App 提供单会话聊天工作区。
function ChatApp({
  messages,
  draft,
  isRunning,
  onDraftChange,
  onSend,
  onCancel,
}: {
  messages: Message[];
  draft: string;
  isRunning: boolean;
  onDraftChange: (value: string) => void;
  onSend: () => void;
  onCancel: () => void;
}) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    textarea.style.height = "auto";
    textarea.style.height = `${Math.min(textarea.scrollHeight, 160)}px`;
  }, [draft]);

  return (
    <section className="agent-page">
      <div className="agent-header">
        <span className={`agent-state ${isRunning ? "busy" : ""}`}>
          <i />
          {isRunning ? "处理中" : "已就绪"}
        </span>
      </div>
      <div className="chat-scroll">
        {messages.length === 0 ? (
          <div className="agent-empty">
            <span className="empty-mark">✦</span>
            <h2>有什么可以帮你？</h2>
            <p>这是 Chat App 的独立会话。</p>
          </div>
        ) : (
          messages.map((message) => (
            <article
              className={`message ${message.role} ${message.status ?? ""}`}
              key={message.id}
            >
              <div className="avatar">
                {message.role === "user" ? "你" : "h"}
              </div>
              <div className="message-body">
                <div className="message-label">
                  {message.role === "user" ? "你" : "Chat Agent"}
                </div>
                <div className="message-content">{message.content}</div>
              </div>
            </article>
          ))
        )}
      </div>
      <form
        className="composer"
        onSubmit={(event) => {
          event.preventDefault();
          onSend();
        }}
      >
        <textarea
          value={draft}
          ref={textareaRef}
          onChange={(event) => onDraftChange(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              onSend();
            }
          }}
          placeholder="输入消息，Shift + Enter 换行"
          rows={1}
        />
        <button
          type={isRunning ? "button" : "submit"}
          onClick={isRunning ? onCancel : undefined}
          disabled={!isRunning && !draft.trim()}
          aria-label={isRunning ? "取消生成" : "发送消息"}
        >
          {isRunning ? "■" : "↑"}
        </button>
      </form>
    </section>
  );
}

// Video App 作为独立 Agent 工作区的占位页面。
function VideoApp() {
  return (
    <section className="agent-page video-page">
      <div className="agent-header">
        <div>
          <span className="eyebrow">VIDEO AGENT</span>
          <h1>视频创作</h1>
        </div>
        <span className="agent-state">
          <i />
          已就绪
        </span>
      </div>
      <div className="video-empty">
        <img className="app-icon" src="/video-icon.png" alt="" />
        <h2>准备开始创作</h2>
        <p>脚本、素材和生成任务将在这里独立管理。</p>
        <button className="primary-button">创建视频任务</button>
      </div>
    </section>
  );
}

// 图片制作 App 展示独立的图片 Agent 工作区入口。
function ImageApp() {
  return (
    <section className="agent-page video-page">
      <div className="agent-header">
        <div>
          <span className="eyebrow">图片制作</span>
          <h1>图片制作</h1>
        </div>
        <span className="agent-state">
          <i />
          已就绪
        </span>
      </div>
      <div className="video-empty">
        <img className="app-icon" src="/image-icon.png" alt="" />
        <h2>准备开始创作</h2>
        <p>提示词、参考图和生成结果将在这里独立管理。</p>
        <button className="primary-button">创建图片任务</button>
      </div>
    </section>
  );
}

// 展示应用配置弹窗并提供 API key 管理。
function SettingsModal({
  settings,
  apiKey,
  showKey,
  onApiKeyChange,
  onToggleKey,
  onClose,
  onSave,
}: {
  settings: SettingsInfo | null;
  apiKey: string;
  showKey: boolean;
  onApiKeyChange: (value: string) => void;
  onToggleKey: () => void;
  onClose: () => void;
  onSave: () => void;
}) {
  return (
    <div
      className="modal-layer"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section
        className="settings-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <div className="modal-header">
          <div>
            <span className="eyebrow">WORKSPACE</span>
            <h2 id="settings-title">连接设置</h2>
          </div>
          <button
            className="close-button"
            aria-label="关闭设置"
            onClick={onClose}
          >
            ×
          </button>
        </div>
        <p className="modal-copy">
          配置 Agnes API key 后即可使用各个 Agent App。
        </p>
        <label className="field-label" htmlFor="api-key">
          API key
        </label>
        <div className="key-input">
          <input
            id="api-key"
            type={showKey ? "text" : "password"}
            value={apiKey}
            onChange={(event) => onApiKeyChange(event.currentTarget.value)}
            placeholder={
              settings?.apiKeyConfigured
                ? "已配置 · 输入新 key 可替换"
                : "请输入 API key"
            }
          />
          <button type="button" className="reveal-button" onClick={onToggleKey}>
            {showKey ? "隐藏" : "显示"}
          </button>
        </div>
        <div className="settings-note">
          {settings?.apiKeyConfigured
            ? "当前已配置 API key"
            : "当前尚未配置 API key"}
        </div>
        <div className="modal-actions">
          <button className="secondary-button" onClick={onClose}>
            取消
          </button>
          <button className="primary-button" onClick={onSave}>
            保存设置
          </button>
        </div>
        <button
          className="register-link"
          onClick={() => void openUrl("https://platform.agnes-ai.cn/login")}
        >
          还没有 API key？<span>免费注册 ↗</span>
        </button>
      </section>
    </div>
  );
}

export default App;

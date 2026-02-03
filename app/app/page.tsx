"use client";
import {invoke} from "@tauri-apps/api/core";
import {useState, useEffect, useRef} from "react";
import {register, unregisterAll} from "@tauri-apps/plugin-global-shortcut";
import {listen} from "@tauri-apps/api/event";

type Tab = "sessions" | "todos" | "knowledge";
type GraphModel = {
    meta: {
        generated_at: string;
    };
    nodes: {
        id: string;
        label: string;
        type: string;
    }[];
    edges: {
        from: string;
        to: string;
        predicate: string;
        confidence: number;
    }[];
};
type KnowledgeRelation = {
    subject: string;
    predicate: string;
    object: string;
    confidence: number;
};

type KnowledgeOverview = {
    persons: string[];
    organizations: string[];
    events: string[];
    relations: KnowledgeRelation[];
};
type TodoItem = {
    id: string;
    kind: string;
    status: string;
    date: string;
    title: string;
    target_type: string;
    target_id: string;
    confidence: number;
    source_document: string;
};
type SessionEntity = {
    entity_type: string;
    value: string;
};
type ProgressState = {
    stage: string;
    message: string;
    percent: number;
    visible: boolean;
};
type Session = {
    id: string;
    date: string;
    title: string;
    raw: string;
    entities: SessionEntity[];
};

function iconFor(type: string) {
    switch (type) {
        case "person":
            return "👤";
        case "organization":
            return "🏢";
        case "location":
            return "📍";
        case "project":
            return "🧩";
        default:
            return "🔹";
    }
}

export default function Home() {
    const iframeRef = useRef<HTMLIFrameElement>(null);
    const [graphModel, setGraphModel] = useState<GraphModel | null>(null);
    const [knowledgeOverview, setKnowledgeOverview] = useState<KnowledgeOverview>({
        persons: [],
        organizations: [],
        events: [],
        relations: [],
    });
    const [todos, setTodos] = useState<TodoItem[]>([]);
    const [todosLoaded, setTodosLoaded] = useState(false);
    const hideTimeoutRef = useRef<number | null>(null);
    const [activeTab, setActiveTab] = useState<Tab>("sessions");
    const [selectedSession, setSelectedSession] = useState<string | null>(null);
    const [pipelineProgress, setPipelineProgress] =
        useState<ProgressState | null>(null);
    const [status, setStatus] = useState<string>("Initialisiere...");
    const [isRecording, setIsRecording] = useState(false);
    const [sessions, setSessions] = useState<Session[]>([]);
    type KnowledgeView = "overview" | "graph";
    const [knowledgeView, setKnowledgeView] = useState<KnowledgeView>("overview");
    const SHORTCUT = "CommandOrControl+Shift+U";
    function refreshKnowledgeGraph() {
        invoke<GraphModel>("get_knowledge_graph")
            .then((model) => {
                console.log("graph model loaded", model);
                setGraphModel(model);
            })
            .catch(console.error);
    }
    useEffect(() => {
        const unlisten = listen<ProgressState>("processing:progress", (event) => {
            const p = event.payload;

            // laufenden Hide-Timer abbrechen
            if (hideTimeoutRef.current) {
                clearTimeout(hideTimeoutRef.current);
                hideTimeoutRef.current = null;
            }

            setPipelineProgress({
                ...p,
                visible: true,
            });

            if (p.percent >= 100) {
                // reload sessions once processing is finished
                invoke<Session[]>("list_sessions")
                    .then(setSessions)
                    .catch(console.error);

                hideTimeoutRef.current = window.setTimeout(() => {
                    setPipelineProgress(null);
                }, 2000);
            }
        });

        return () => {
            unlisten.then(f => f());
            if (hideTimeoutRef.current) {
                clearTimeout(hideTimeoutRef.current);
            }
        };
    }, []);
    useEffect(() => {
        if (!graphModel) return;
        if (!iframeRef.current?.contentWindow) return;

        console.log("model updated, pushing to iframe");

        iframeRef.current.contentWindow.postMessage(
            {
                type: "KNOWLEDGE_GRAPH",
                payload: graphModel,
            },
            "*"
        );
    }, [graphModel]);

    useEffect(() => {
        if (activeTab === "knowledge" && knowledgeView === "graph") {
            refreshKnowledgeGraph();
        }
    }, [activeTab, knowledgeView]);
    useEffect(() => {
        if (activeTab !== "sessions") return;
        if (!selectedSession) return;

        requestAnimationFrame(() => {
            document.getElementById(`session-${selectedSession}`)?.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
        });
    }, [activeTab, selectedSession, sessions.length]);

    useEffect(() => {
        document.title = isRecording ? "VIA ● REC" : "VIA";
    }, [isRecording]);
    useEffect(() => {
        if (activeTab !== "todos") return;
        if (todosLoaded) return;

        invoke<TodoItem[]>("list_todos")
            .then((data) => {
                setTodos(data);
                setTodosLoaded(true);
            })
            .catch(console.error);
    }, [activeTab, setTodos, todosLoaded]);
    useEffect(() => {
        invoke<Session[]>("list_sessions")
            .then(setSessions)
            .catch(console.error);
    }, []);
    useEffect(() => {
        invoke<TodoItem[]>("list_todos")
            .then(setTodos)
            .catch(console.error);
    }, []);
    useEffect(() => {
        let isMounted = true;

        async function setupShortcut() {
            try {
                await unregisterAll();
                await register(SHORTCUT, async (event) => {
                    if (event.state !== "Pressed") return;
                    try {
                        const backendIsRecording = await invoke<boolean>("recorder_status");
                        const timestamp = new Date().toLocaleTimeString();

                        if (!backendIsRecording) {
                            await invoke("start_recording").catch(async (e) => {
                                console.error(e);
                                const s = await invoke<boolean>("recorder_status");
                                setIsRecording(s);
                                throw e;
                            });

                            setIsRecording(true);
                            setStatus(`● Aufnahme gestartet um ${timestamp}`);
                        } else {
                            const audioPath = await invoke<string>("stop_recording");

                            setIsRecording(false);
                            setStatus(`■ Aufnahme beendet um ${timestamp}`);
                            invoke("process_recording", { audioPath })
                                .then(() => invoke<Session[]>("list_sessions"))
                                .then(setSessions)
                                .catch(console.error);

                            // await invoke("stop_recording");
                            // const sessions = await invoke<Session[]>("list_sessions");
                            // setSessions(sessions);
                            // setIsRecording(false);
                        }
                    } catch (e) {
                        console.error(e);
                    }
                });

                if (isMounted) {
                    setStatus(`Bereit! ${SHORTCUT}`);
                }
            } catch {
                if (isMounted) setStatus("Hotkey-Fehler");
            }
        }

        setupShortcut();

        return () => {
            isMounted = false;
            unregisterAll();
        };
    }, []);

    function jumpToSession(recordId?: string) {
        if (!recordId) return;
        setActiveTab("sessions");
        setSelectedSession(recordId);
    }

    function refreshTodos() {
        invoke<TodoItem[]>("list_todos")
            .then(setTodos)
            .catch(console.error);
    }

    function refreshKnowledgeOverview() {
        invoke<KnowledgeOverview>("get_knowledge_overview")
            .then(setKnowledgeOverview)
            .catch(console.error);
    }

    useEffect(() => {
        if (activeTab !== "knowledge") return;

        refreshKnowledgeOverview();
    }, [activeTab]);

    const currentSession = sessions.find(s => s.id === selectedSession);

    return (
        <main className="min-h-screen bg-gray-900 text-white p-6">
            {/* Header */}
            <header className="mb-6">
                <h1 className="text-3xl font-bold text-blue-400">
                    VIA
                </h1>
                <p className="text-gray-400 text-sm">
                    Personal Voice Intelligence · {SHORTCUT} {selectedSession}
                </p>
            </header>
            {pipelineProgress && (
                <div className="fixed bottom-4 left-4 right-4 bg-gray-800 border border-gray-600 rounded p-3">
                    <div className="text-xs text-gray-400 mb-1">
                        {pipelineProgress.stage}
                    </div>

                    <div className="text-sm mb-2">
                        {pipelineProgress.message}
                    </div>

                    <div className="w-full bg-gray-700 h-2 rounded">
                        <div
                            className="bg-blue-500 h-2 rounded transition-all"
                            style={{ width: `${pipelineProgress.percent}%` }}
                        />
                    </div>
                </div>
            )}
            {/* Status */}
            <div className="flex items-center gap-4 mb-6">
                <div
                    className={`w-3 h-3 rounded-full ${
                        isRecording ? "bg-red-500 animate-pulse" : "bg-gray-500"
                    }`}
                />
                <span className="text-sm">{status}</span>
            </div>

            {/* Tabs */}
            <div className="flex gap-4 border-b border-gray-700 mb-6">
                <button onClick={() => setActiveTab("sessions")}
                        className={activeTab === "sessions" ? "border-b-2 border-blue-400 pb-2" : "pb-2 text-gray-400"}>
                    Sessions
                </button>
                <button onClick={() => setActiveTab("todos")}
                        className={activeTab === "todos" ? "border-b-2 border-blue-400 pb-2" : "pb-2 text-gray-400"}>
                    To-Dos
                </button>
                <button onClick={() => setActiveTab("knowledge")}
                        className={activeTab === "knowledge" ? "border-b-2 border-blue-400 pb-2" : "pb-2 text-gray-400"}>
                    Wissen
                </button>
            </div>

            {/* Content */}
            {activeTab === "sessions" && (
                <div className="grid grid-cols-3 gap-6">
                    <div className="col-span-1 border border-gray-700 rounded p-4">
                        {sessions.map((s) => {
                            const selected = s.id === selectedSession;

                            return (
                                <div
                                    key={s.id}
                                    id={`session-${s.id}`}
                                    onClick={() => setSelectedSession(s.id)}
                                    className={[
                                        "cursor-pointer mb-3 p-2 rounded",
                                        selected ? "bg-blue-900/40 border border-blue-500/40" : "hover:bg-gray-800",
                                    ].join(" ")}
                                >
                                    <div className="font-semibold">{s.title}</div>
                                    <div className="text-xs text-gray-400">{s.date}</div>
                                </div>
                            );
                        })}
                    </div>

                    <div className="col-span-2 border border-gray-700 rounded p-4">
                        {!currentSession && (
                            <p className="text-gray-500">Session auswählen</p>
                        )}

                        {currentSession && (
                            <>
                                <h2 className="text-xl font-bold mb-2">Ergebnis</h2>
                                <p className="mb-4">summary</p>
                                currentSession.summary

                                <details className="text-sm text-gray-300">
                                    <summary className="cursor-pointer mb-2">Rohtranskript</summary>
                                    <pre className="whitespace-pre-wrap">{currentSession.raw}</pre>
                                </details>
                            </>
                        )}
                        {currentSession && (
                            <>
                                <h2 className="text-xl font-bold mb-2">Ergebnis</h2>

                                <div className="flex flex-wrap gap-2 mb-4 text-xs">
                                    {currentSession.entities.map(e => (
                                        <span
                                            key={`${e.entity_type}:${e.value}`}
                                            className="px-2 py-1 rounded bg-gray-700 text-xs"
                                        >
                                        {iconFor(e.entity_type)} {e.value}
                                      </span>
                                    ))}
                                </div>

                                <p className="mb-4">summary</p>
                            </>
                        )}
                    </div>
                </div>
            )}

            {activeTab === "todos" && (
                <div className="space-y-4">
                    {todos.length === 0 && (
                        <p className="text-gray-500">Keine To-Dos vorhanden</p>
                    )}

                    {todos.map(todo => (
                        <div key={todo.id}
                             className="border border-gray-700 rounded p-4 flex justify-between items-center">
                            <div>
                                <div className="font-medium">{todo.title}</div>
                                <div className="text-xs text-gray-400">
                                    {todo.kind} · {todo.status}
                                </div>
                            </div>
                            <div className="flex gap-2">
                                <button
                                    onClick={() => jumpToSession(todo.source_document)}
                                    className="px-3 py-1 bg-blue-600 rounded text-sm"
                                >
                                    Zur Session
                                </button>

                                <button className="px-3 py-1 bg-green-600 rounded text-sm"
                                        onClick={() =>
                                            invoke("confirm_todo", {knowledgeId: todo.id})
                                                .then(() => refreshTodos())
                                        }
                                >
                                    Bestätigen
                                </button>

                                <button className="px-3 py-1 bg-gray-700 rounded text-sm"
                                        onClick={() =>
                                            invoke("ignore_todo", {knowledgeId: todo.id})
                                                .then(() => refreshTodos())
                                        }
                                >
                                    Ignorieren
                                </button>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            {activeTab === "knowledge" && (
                <div className="w-full h-full flex flex-col gap-4">
                    {/* Sub Tabs */}
                    <div className="flex gap-4 border-b border-gray-700">
                        <button
                            onClick={() => setKnowledgeView("overview")}
                            className={
                                knowledgeView === "overview"
                                    ? "border-b-2 border-blue-400 pb-2"
                                    : "pb-2 text-gray-400"
                            }
                        >
                            Übersicht
                        </button>

                        <button
                            onClick={() => setKnowledgeView("graph")}
                            className={
                                knowledgeView === "graph"
                                    ? "border-b-2 border-blue-400 pb-2"
                                    : "pb-2 text-gray-400"
                            }
                        >
                            Graph
                        </button>
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-h-0 ">
                        {knowledgeView === "overview" && (
                            <div className="text-sm text-gray-300 space-y-4">
                                <div>
                                    <h3>Personen</h3>
                                    {knowledgeOverview.persons.map(p => (
                                        <div key={p}>👤 {p}</div>
                                    ))}
                                </div>

                                <div>
                                    <h3>Organisationen</h3>
                                    {knowledgeOverview.organizations.map(o => (
                                        <div key={o}>🏢 {o}</div>
                                    ))}
                                </div>

                                <div>
                                    <h3>Relationen</h3>
                                    {knowledgeOverview.relations.map((r, i) => (
                                        <div key={i}>
                                            {r.subject} {r.predicate} {r.object}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {knowledgeView === "graph" && (
                            <div className="relative flex-1 min-h-0">
                            <iframe
                                ref={iframeRef}
                                src="/graph.html"
                                style={{height: '320px'}}
                                className="w-full h-full border-0"
                                onLoad={() => {
                                    if (!graphModel) {
                                        console.log("iframe loaded, but no model yet");
                                        return;
                                    }

                                    console.log("iframe loaded, sending model");

                                    iframeRef.current?.contentWindow?.postMessage(
                                        {
                                            type: "KNOWLEDGE_GRAPH",
                                            payload: graphModel,
                                        },
                                        "*"
                                    );
                                }}
                            />
                            </div>
                        )}
                    </div>
                </div>
            )}
        </main>
    );
}


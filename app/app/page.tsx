"use client";
import {invoke} from "@tauri-apps/api/core";
import {useState, useEffect} from "react";
import {register, unregisterAll} from "@tauri-apps/plugin-global-shortcut";

type Tab = "sessions" | "todos" | "knowledge";

type Session = {
    id: string;
    date: string;
    title: string;
    raw: string;
    persons: string[];
    organizations: string[];
    locations: string[];
    projects: string[];
};

const mockTodos = [
    {
        id: "1",
        text: "Beziehung zwischen Matthias Ebel-Koch und Edo Logic GmbH bestätigen",
        type: "frage"
    },
    {
        id: "2",
        text: "Projekt Einfach Wuhu weiter ausarbeiten",
        type: "aufgabe"
    }
];

export default function Home() {
    const [activeTab, setActiveTab] = useState<Tab>("sessions");
    const [selectedSession, setSelectedSession] = useState<string | null>(null);

    const [logs, setLogs] = useState<string[]>([]);
    const [status, setStatus] = useState<string>("Initialisiere...");
    const [isRecording, setIsRecording] = useState(false);
    const [sessions, setSessions] = useState<Session[]>([]);
    type KnowledgeView = "overview" | "graph";

    const [knowledgeView, setKnowledgeView] = useState<KnowledgeView>("overview");
    const SHORTCUT = "CommandOrControl+Shift+U";

    useEffect(() => {
        document.title = isRecording ? "VIA ● REC" : "VIA";
    }, [isRecording]);

    useEffect(() => {
        invoke<Session[]>("list_sessions")
            .then(setSessions)
            .catch(console.error);
    }, []);

    useEffect(() => {
        let isMounted = true;

        async function setupShortcut() {
            try {
                await unregisterAll();

                await register(SHORTCUT, (event) => {
                    if (event.state !== "Pressed") return;

                    setIsRecording((prev) => {
                        const nextState = !prev;
                        const timestamp = new Date().toLocaleTimeString();

                        if (nextState) {
                            invoke("start_recording").catch(() => {
                            });
                        } else {
                            invoke("stop_recording").catch(() => {
                            });
                        }

                        const message = nextState
                            ? `● Aufnahme gestartet um ${timestamp}`
                            : `■ Aufnahme beendet um ${timestamp}`;

                        setLogs((prevLogs) => [message, ...prevLogs]);
                        setStatus(nextState ? "Aufnahme läuft…" : `Bereit!`);

                        return nextState;
                    });
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

    const currentSession = sessions.find(s => s.id === selectedSession);

    return (
        <main className="min-h-screen bg-gray-900 text-white p-6">
            {/* Header */}
            <header className="mb-6">
                <h1 className="text-3xl font-bold text-blue-400">
                    VIA
                </h1>
                <p className="text-gray-400 text-sm">
                    Personal Voice Intelligence · {SHORTCUT}
                </p>
            </header>

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
                        {sessions.map(s => (
                            <div
                                key={s.id}
                                onClick={() => setSelectedSession(s.id)}
                                className="cursor-pointer mb-3 p-2 rounded hover:bg-gray-800"
                            >
                                <div className="font-semibold">{s.title}</div>
                                <div className="text-xs text-gray-400">{s.date}</div>
                            </div>
                        ))}
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
                                    {currentSession.persons.map(p => (
                                        <span key={p} className="px-2 py-1 rounded bg-blue-800 text-blue-200">
                                          👤 {p}
                                        </span>
                                    ))}
                                    {currentSession.organizations.map(o => (
                                        <span key={o} className="px-2 py-1 rounded bg-green-800 text-green-200">
                                          🏢 {o}
                                        </span>
                                    ))}
                                    {currentSession.locations.map(l => (
                                        <span key={l} className="px-2 py-1 rounded bg-yellow-800 text-yellow-200">
                                          📍 {l}
                                        </span>
                                    ))}

                                    {currentSession.projects.map((p) => (
                                        <span
                                            key={`project-${p}`}
                                            className="px-2 py-1 text-xs rounded bg-amber-600 text-black"
                                        >
                                            🧩 {p}
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
                    {mockTodos.map(todo => (
                        <div key={todo.id}
                             className="border border-gray-700 rounded p-4 flex justify-between items-center">
                            <div>
                                <div className="font-medium">{todo.text}</div>
                                <div className="text-xs text-gray-400">{todo.type}</div>
                            </div>
                            <div className="flex gap-2">
                                <button className="px-3 py-1 bg-green-600 rounded text-sm">Bestätigen</button>
                                <button className="px-3 py-1 bg-gray-700 rounded text-sm">Ignorieren</button>
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
                    <div className="flex-1 min-h-0">
                        {knowledgeView === "overview" && (
                            <div className="text-gray-400">
                                Wissensübersicht folgt.
                                <br />
                                Personen, Organisationen, Projekte, Orte.
                            </div>
                        )}

                        {knowledgeView === "graph" && (
                            <iframe
                                src="/graph.html"
                                className="w-full h-full border-0"
                            />
                        )}
                    </div>
                </div>
            )}
        </main>
    );
}


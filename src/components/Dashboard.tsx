import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface GalleryInfo {
    id: string;
    name: string;
}

interface ProgressEvent {
    current: number;
    total: number;
    message: string;
}

export function Dashboard() {
    const [galleries, setGalleries] = useState<GalleryInfo[]>([]);
    const [postType, setPostType] = useState<"posting" | "comment">("posting");
    const [selectedGallery, setSelectedGallery] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [status, setStatus] = useState("대기 중");
    const [isRunning, setIsRunning] = useState(false);
    const [progress, setProgress] = useState({ current: 0, total: 0 });
    const [captchaKey, setCaptchaKey] = useState("");
    const [captchaType, setCaptchaType] = useState<"2captcha" | "anticaptcha">("2captcha");

    useEffect(() => {
        const unlisten = listen<ProgressEvent>("cleaning_progress", (event) => {
            setProgress({ current: event.payload.current, total: event.payload.total });
            setStatus(event.payload.message);
            if (event.payload.current >= event.payload.total && event.payload.total > 0) {
                setIsRunning(false);
            }
        });
        return () => {
            unlisten.then(fn => fn());
        };
    }, []);

    const fetchGalleries = async () => {
        setLoading(true);
        try {
            const res = await invoke<GalleryInfo[]>("get_galleries", { postType });
            setGalleries(res);
        } catch (e) {
            console.error(e);
            setStatus("갤러리 목록을 불러오는데 실패했습니다");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchGalleries();
    }, [postType]);

    const handleStart = async () => {
        setStatus("시작 중...");
        setIsRunning(true);
        setProgress({ current: 0, total: 0 });
        try {
            const res = await invoke<string>("start_cleaning", {
                postType,
                galleryId: selectedGallery,
                captchaKey: captchaKey || null,
                captchaType: captchaType || null
            });
            setStatus(res);
        } catch (e: any) {
            setStatus("오류: " + e.toString());
        } finally {
            setIsRunning(false);
        }
    };

    const getSelectedGalleryName = () => {
        if (selectedGallery === null) return "전체";
        return galleries.find(g => g.id === selectedGallery)?.name || "";
    };

    const progressPercent = progress.total > 0 ? Math.round((progress.current / progress.total) * 100) : 0;

    return (
        <div className="min-h-screen bg-white p-6">
            <div className="flex items-center justify-between mb-8">
                <h1 className="text-xl font-semibold text-gray-900">DC Remover</h1>
                <div className="flex gap-1 bg-gray-100 p-1 rounded-lg">
                    <button
                        onClick={() => setPostType("posting")}
                        className={`px-4 py-1.5 text-sm rounded-md ${postType === "posting" ? "bg-white text-gray-900" : "text-gray-500"}`}
                    >
                        게시글
                    </button>
                    <button
                        onClick={() => setPostType("comment")}
                        className={`px-4 py-1.5 text-sm rounded-md ${postType === "comment" ? "bg-white text-gray-900" : "text-gray-500"}`}
                    >
                        댓글
                    </button>
                </div>
            </div>

            <div className="flex gap-8">
                <div className="flex-1">
                    <div className="flex items-center gap-2 mb-3">
                        <span className="text-sm text-gray-500">갤러리</span>
                        {loading && <span className="text-xs text-gray-400">로딩 중...</span>}
                    </div>
                    <div className="space-y-1 max-h-[500px] overflow-y-auto">
                        <button
                            onClick={() => setSelectedGallery(null)}
                            className={`w-full text-left px-3 py-2 rounded-lg text-sm ${selectedGallery === null ? "bg-gray-900 text-white" : "text-gray-700 hover:bg-gray-100"}`}
                        >
                            전체 갤러리
                        </button>
                        {galleries.map(g => (
                            <button
                                key={g.id}
                                onClick={() => setSelectedGallery(g.id)}
                                className={`w-full text-left px-3 py-2 rounded-lg text-sm ${selectedGallery === g.id ? "bg-gray-900 text-white" : "text-gray-700 hover:bg-gray-100"}`}
                            >
                                {g.name}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="w-72 space-y-6">
                    <div>
                        <span className="text-sm text-gray-500">선택됨</span>
                        <p className="text-lg font-medium text-gray-900">{getSelectedGalleryName()}</p>
                    </div>

                    <div className="space-y-2">
                        <span className="text-sm text-gray-500">캡차 (선택)</span>
                        <input
                            type="text"
                            value={captchaKey}
                            onChange={(e) => setCaptchaKey(e.target.value)}
                            placeholder="API Key"
                            className="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:border-gray-400"
                        />
                        <div className="flex gap-2">
                            <button
                                onClick={() => setCaptchaType("2captcha")}
                                className={`flex-1 py-1.5 text-sm rounded-md border ${captchaType === "2captcha" ? "bg-gray-900 text-white border-gray-900" : "border-gray-200 text-gray-600"}`}
                            >
                                2Captcha
                            </button>
                            <button
                                onClick={() => setCaptchaType("anticaptcha")}
                                className={`flex-1 py-1.5 text-sm rounded-md border ${captchaType === "anticaptcha" ? "bg-gray-900 text-white border-gray-900" : "border-gray-200 text-gray-600"}`}
                            >
                                AntiCaptcha
                            </button>
                        </div>
                    </div>

                    {isRunning && progress.total > 0 && (
                        <div className="space-y-2">
                            <div className="flex justify-between text-sm text-gray-600">
                                <span>진행률</span>
                                <span>{progress.current} / {progress.total} ({progressPercent}%)</span>
                            </div>
                            <div className="w-full bg-gray-200 rounded-full h-2">
                                <div
                                    className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                                    style={{ width: `${progressPercent}%` }}
                                />
                            </div>
                        </div>
                    )}

                    <button
                        onClick={handleStart}
                        disabled={isRunning}
                        className={`w-full py-3 rounded-lg font-medium ${isRunning ? "bg-gray-200 text-gray-400" : "bg-red-500 text-white hover:bg-red-600"}`}
                    >
                        {isRunning ? "처리 중..." : "삭제 시작"}
                    </button>

                    <div className="flex items-center gap-2 text-sm text-gray-500">
                        <span className={`w-2 h-2 rounded-full ${isRunning ? "bg-yellow-400 animate-pulse" : "bg-green-400"}`} />
                        {status}
                    </div>
                </div>
            </div>
        </div>
    );
}

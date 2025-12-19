import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LoginPageProps {
    onLoginSuccess: () => void;
}

export function LoginPage({ onLoginSuccess }: LoginPageProps) {
    const [id, setId] = useState("");
    const [pw, setPw] = useState("");
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleLogin = async () => {
        if (!id || !pw) {
            setError("아이디와 비밀번호를 입력해주세요");
            return;
        }

        setLoading(true);
        setError(null);
        try {
            const success = await invoke<boolean>("login", { id, pw });
            if (success) {
                onLoginSuccess();
            } else {
                setError("로그인 실패. 아이디/비밀번호를 확인해주세요.");
            }
        } catch (e: any) {
            setError("오류: " + e.toString());
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-white flex items-center justify-center p-6">
            <div className="w-full max-w-xs">
                <h1 className="text-xl font-semibold text-gray-900 text-center mb-8">DC Remover</h1>

                <div className="space-y-4">
                    <div className="space-y-1">
                        <label className="text-sm text-gray-600">아이디</label>
                        <input
                            type="text"
                            value={id}
                            onChange={(e) => setId(e.target.value)}
                            placeholder="디시인사이드 아이디"
                            className="w-full px-3 py-2.5 border border-gray-200 rounded-lg text-sm focus:outline-none focus:border-gray-400"
                        />
                    </div>

                    <div className="space-y-1">
                        <label className="text-sm text-gray-600">비밀번호</label>
                        <input
                            type="password"
                            value={pw}
                            onChange={(e) => setPw(e.target.value)}
                            onKeyDown={(e) => e.key === "Enter" && handleLogin()}
                            placeholder="비밀번호"
                            className="w-full px-3 py-2.5 border border-gray-200 rounded-lg text-sm focus:outline-none focus:border-gray-400"
                        />
                    </div>

                    {error && (
                        <p className="text-sm text-red-500">{error}</p>
                    )}

                    <button
                        onClick={handleLogin}
                        disabled={loading}
                        className={`w-full py-2.5 rounded-lg font-medium text-sm ${loading ? "bg-gray-200 text-gray-400" : "bg-gray-900 text-white hover:bg-gray-800"}`}
                    >
                        {loading ? "로그인 중..." : "로그인"}
                    </button>
                </div>
            </div>
        </div>
    );
}

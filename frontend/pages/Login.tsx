import { useState } from 'react';
import { motion } from 'framer-motion';
import { Link, useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';

export default function Login() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const { login, isLoading, error, clearError } = useAuthStore();
  const navigate = useNavigate();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    clearError();
    try {
      await login(username, password);
      navigate('/');
    } catch (err) {
      // Error is handled in the store
    }
  };

  return (
    <div className="h-screen w-full bg-slate-950 flex items-center justify-center p-6 relative overflow-hidden">
      {/* Background Decor */}
      <div className="absolute top-0 left-0 w-full h-full opacity-30 pointer-events-none">
        <div className="absolute top-[-10%] right-[-10%] w-[500px] h-[500px] bg-blue-600/20 rounded-full blur-[120px]"></div>
        <div className="absolute bottom-[-10%] left-[-10%] w-[500px] h-[500px] bg-indigo-600/20 rounded-full blur-[120px]"></div>
      </div>

      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="w-full max-w-md z-10"
      >
        <div className="flex flex-col items-center mb-12">
          <div className="w-16 h-16 bg-blue-600 rounded-2xl flex items-center justify-center font-black text-3xl text-white shadow-2xl shadow-blue-500/50 mb-6">N</div>
          <h1 className="text-4xl font-black tracking-tight text-white mb-2">NEXUS<span className="text-blue-500">TASK</span></h1>
          <p className="text-slate-500 font-medium">Inteligencia de Proyectos Empresariales</p>
        </div>

        <div className="bg-slate-900/80 border border-slate-800 p-10 rounded-[40px] shadow-2xl backdrop-blur-2xl">
          <h2 className="text-2xl font-bold text-white mb-2">Autenticación Requerida</h2>
          <p className="text-slate-500 text-sm mb-8">Inicie sesión para acceder a su ecosistema.</p>

          {error && (
            <div className="mb-6 p-4 bg-red-500/10 border border-red-500/50 rounded-xl text-red-400 text-sm">
              {error}
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label className="block text-[11px] font-bold text-slate-500 uppercase tracking-widest mb-2 ml-1">Usuario</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full bg-slate-800/50 border border-slate-700 rounded-2xl px-5 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                placeholder="usuario_id"
                required
              />
            </div>

            <div>
              <label className="block text-[11px] font-bold text-slate-500 uppercase tracking-widest mb-2 ml-1">Contraseña</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full bg-slate-800/50 border border-slate-700 rounded-2xl px-5 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                placeholder="••••••••••••"
                required
              />
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-50 hover:to-indigo-500 text-white font-bold py-4 rounded-2xl shadow-xl shadow-blue-900/40 transition-all active:scale-95 flex items-center justify-center gap-2 mt-4 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? 'Autenticando...' : 'Iniciar Sesión Segura'}
            </button>

            <div className="pt-6 text-center">
              <Link to="/register" className="text-xs text-slate-500 hover:text-blue-400 transition-colors">
                ¿No tiene cuenta? Registrarse
              </Link>
            </div>
          </form>
        </div>

        <p className="mt-12 text-center text-[10px] text-slate-600 uppercase tracking-[0.2em] font-bold">
          v2.0.0-enterprise | Protocolo Seguro v4.2
        </p>
      </motion.div>
    </div>
  );
}

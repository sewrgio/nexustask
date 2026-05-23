import { useAuthStore } from '../store/authStore';
import { LogOut } from 'lucide-react';

export default function Settings() {
  const { user, logout } = useAuthStore();

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-white">Ajustes</h2>
        <p className="text-slate-500 text-sm">Configuración de usuario y preferencias</p>
      </div>

      <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-8 backdrop-blur-xl">
        <div className="space-y-6">
          <div>
            <h3 className="text-lg font-bold text-white mb-4">Información de Usuario</h3>
            <div className="space-y-3">
              <div>
                <p className="text-xs font-bold text-slate-500 uppercase tracking-widest mb-1">Usuario</p>
                <p className="text-white">{user?.username}</p>
              </div>
              <div>
                <p className="text-xs font-bold text-slate-500 uppercase tracking-widest mb-1">Email</p>
                <p className="text-white">{user?.email}</p>
              </div>
              <div>
                <p className="text-xs font-bold text-slate-500 uppercase tracking-widest mb-1">Rol</p>
                <p className="text-white">{user?.role}</p>
              </div>
            </div>
          </div>

          <div className="pt-6 border-t border-slate-800">
            <button
              onClick={logout}
              className="w-full bg-red-600/10 hover:bg-red-600/20 text-red-400 border border-red-600/50 px-4 py-3 rounded-xl font-bold flex items-center justify-center gap-2 transition-all"
            >
              <LogOut size={18} />
              Cerrar Sesión
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

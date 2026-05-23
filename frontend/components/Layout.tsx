import { Outlet, Link, useLocation } from 'react-router-dom';
import { LayoutDashboard, CheckSquare, Kanban, BarChart3, Settings, ShieldAlert, LogOut, Search, Plus, Bell, User, Users } from 'lucide-react';
import { useAuthStore } from '../store/authStore';

export default function Layout() {
  const location = useLocation();
  const { user, logout } = useAuthStore();
  
  const getActiveModule = () => {
    if (location.pathname === '/') return 'dashboard';
    if (location.pathname === '/tasks') return 'tasks';
    if (location.pathname === '/workspaces') return 'workspaces';
    if (location.pathname === '/kanban') return 'kanban';
    if (location.pathname === '/reports') return 'reports';
    if (location.pathname === '/admin') return 'admin';
    if (location.pathname === '/settings') return 'settings';
    return 'dashboard';
  };

  const activeModule = getActiveModule();

  return (
    <div className="flex h-screen w-full bg-slate-950 text-slate-200 overflow-hidden">
      {/* Sidebar */}
      <aside className="w-64 bg-slate-900/50 border-r border-slate-800 flex flex-col backdrop-blur-xl">
        <div className="p-6 flex items-center gap-3">
          <div className="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center font-bold text-white">N</div>
          <h1 className="text-xl font-bold tracking-tight text-white">NEXUSTASK</h1>
        </div>

        <nav className="flex-1 px-4 py-4 space-y-1">
          <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest px-2 mb-4">Menú Principal</p>
          <NavItem icon={<LayoutDashboard size={18}/>} label="Panel Control" active={activeModule === "dashboard"} to="/" />
          <NavItem icon={<CheckSquare size={18}/>} label="Árbol Tareas" active={activeModule === "tasks"} to="/tasks" />
          <NavItem icon={<Users size={18}/>} label="Espacios" active={activeModule === "workspaces"} to="/workspaces" />
          <NavItem icon={<Kanban size={18}/>} label="Tablero Kanban" active={activeModule === "kanban"} to="/kanban" />
          <NavItem icon={<BarChart3 size={18}/>} label="Analíticas" active={activeModule === "reports"} to="/reports" />
          
          <div className="pt-8 pb-4">
             <p className="text-[10px] font-bold text-slate-500 uppercase tracking-widest px-2 mb-4">Sistema</p>
             <NavItem icon={<ShieldAlert size={18}/>} label="Panel Admin" active={activeModule === "admin"} to="/admin" />
             <NavItem icon={<Settings size={18}/>} label="Ajustes" active={activeModule === "settings"} to="/settings" />
          </div>
        </nav>

        <div className="p-4 border-t border-slate-800 bg-slate-900/80">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center font-bold text-white shadow-lg">
              {user?.username?.[0]?.toUpperCase() || 'U'}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-bold text-white truncate">{user?.username || 'Usuario'}</p>
              <p className="text-[10px] text-slate-500">{user?.role || 'Miembro'}</p>
            </div>
            <button onClick={logout} className="p-2 hover:bg-slate-800 rounded-lg transition-colors text-slate-400">
              <LogOut size={16} />
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-w-0 bg-[radial-gradient(ellipse_at_top_right,_var(--tw-gradient-stops))] from-slate-900 via-slate-950 to-slate-950">
        <header className="h-16 border-b border-slate-800 flex items-center justify-between px-8 backdrop-blur-md bg-slate-950/50 sticky top-0 z-10">
          <div className="flex items-center gap-4 flex-1 max-w-xl">
             <div className="relative w-full">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={16} />
                <input 
                  type="text" 
                  placeholder="Buscar tareas, inteligencia, archivos..." 
                  className="w-full bg-slate-900/50 border border-slate-800 rounded-full py-1.5 pl-10 pr-4 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all placeholder:text-slate-600"
                />
             </div>
          </div>
          
          <div className="flex items-center gap-3">
            <button className="p-2 text-slate-400 hover:text-white transition-colors relative">
               <Bell size={20} />
               <span className="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full border-2 border-slate-950"></span>
            </button>
            <Link to="/tasks" className="bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-bold flex items-center gap-2 transition-all shadow-lg shadow-blue-900/20 active:scale-95">
              <Plus size={18} />
              Nueva Tarea
            </Link>
          </div>
        </header>

        <div className="flex-1 overflow-y-auto p-8">
          <Outlet />
        </div>
      </main>
    </div>
  );
}

function NavItem({ icon, label, active, to }: { icon: any, label: string, active: boolean, to: string }) {
  return (
    <Link 
      to={to}
      className={`w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium transition-all ${active ? 'bg-blue-600 text-white shadow-lg shadow-blue-900/20' : 'text-slate-400 hover:text-white hover:bg-slate-800/50'}`}
    >
      {icon}
      {label}
    </Link>
  );
}

import { NavLink } from 'react-router-dom'
import {
  LayoutDashboard,
  Server,
  Terminal,
  ScrollText,
  Puzzle,
  Network,
  HardDrive,
  CalendarClock,
  Settings,
  Activity,
  FolderSync,
  CalendarDays,
} from 'lucide-react'
import logo from '../../assets/logo/logo-128.png'

const navItems = [
  { to: '/dashboard',  icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/servers',    icon: Server,           label: 'Servidores' },
  { to: '/cluster',    icon: Network,          label: 'Cluster' },
  { to: '/monitoring', icon: Activity,          label: 'Monitoramento' },
  { to: '/backups',    icon: HardDrive,        label: 'Backups' },
  { to: '/scheduler',  icon: CalendarClock,    label: 'Agendador' },
  { to: '/sync',       icon: FolderSync,    label: 'Sincronização' },
  { to: '/events',     icon: CalendarDays,  label: 'Eventos' },
  { to: '/settings',   icon: Settings,      label: 'Configurações' },
]

const serverItems = [
  { to: '/rcon/0',  icon: Terminal,   label: 'RCON' },
  { to: '/logs/0',  icon: ScrollText, label: 'Logs' },
  { to: '/mods/0',  icon: Puzzle,     label: 'Mods' },
]

export default function Sidebar() {
  return (
    <aside className="flex flex-col w-56 min-w-56 h-full bg-surface-900 border-r border-surface-700">
      {/* Logo */}
      <div className="flex flex-col items-center gap-2 px-4 py-5 border-b border-surface-700">
        <img src={logo} alt="Ark Manager" className="w-16 h-16 object-contain" />
        <span className="font-semibold text-slate-100 text-sm tracking-wide">
          Ark Manager
        </span>
      </div>

      {/* Navegação principal */}
      <nav className="flex-1 overflow-y-auto scrollbar-thin py-3 px-2 space-y-0.5">
        {navItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? 'bg-ark-600 text-white font-medium'
                  : 'text-slate-400 hover:text-slate-100 hover:bg-surface-800'
              }`
            }
          >
            <Icon size={16} strokeWidth={1.75} />
            {label}
          </NavLink>
        ))}

        <div className="pt-3 pb-1 px-3">
          <p className="text-xs text-slate-500 uppercase tracking-wider font-medium">
            Servidor ativo
          </p>
        </div>

        {serverItems.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? 'bg-ark-600 text-white font-medium'
                  : 'text-slate-400 hover:text-slate-100 hover:bg-surface-800'
              }`
            }
          >
            <Icon size={16} strokeWidth={1.75} />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Versão */}
      <div className="px-4 py-3 border-t border-surface-700">
        <p className="text-xs text-slate-600">v1.0.0</p>
      </div>
    </aside>
  )
}

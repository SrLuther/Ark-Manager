import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { format, formatDistanceToNow, parseISO } from 'date-fns'
import { ptBR } from 'date-fns/locale'
import type { ArkMap, ServerStatus } from '../types'

/** Combina classes CSS com Tailwind merge */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Formata bytes em string legível (B, KB, MB, GB) */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

/** Formata data ISO para dd/MM/yyyy HH:mm */
export function formatDate(iso: string): string {
  try {
    return format(parseISO(iso), 'dd/MM/yyyy HH:mm', { locale: ptBR })
  } catch {
    return iso
  }
}

/** Formata data ISO como "há X tempo" */
export function formatRelative(iso: string): string {
  try {
    return formatDistanceToNow(parseISO(iso), { locale: ptBR, addSuffix: true })
  } catch {
    return iso
  }
}

/** Retorna cor Tailwind para o status do servidor */
export function statusColor(status: ServerStatus): string {
  switch (status) {
    case 'running':    return 'text-emerald-400'
    case 'starting':   return 'text-yellow-400'
    case 'stopping':   return 'text-orange-400'
    case 'updating':   return 'text-blue-400'
    case 'installing': return 'text-purple-400'
    case 'error':      return 'text-red-400'
    default:           return 'text-slate-400'
  }
}

/** Retorna label pt-BR do status */
export function statusLabel(status: ServerStatus): string {
  const labels: Record<ServerStatus, string> = {
    running:    'Rodando',
    starting:   'Iniciando',
    stopping:   'Parando',
    stopped:    'Parado',
    error:      'Erro',
    updating:   'Atualizando',
    installing: 'Instalando',
  }
  return labels[status] ?? status
}

/** Retorna label legível do mapa */
export function mapLabel(map: ArkMap): string {
  const labels: Record<ArkMap, string> = {
    TheIsland:     'The Island',
    ScorchedEarth: 'Scorched Earth',
    Aberration:    'Aberration',
    Extinction:    'Extinction',
    Genesis:       'Genesis Part 1',
    Genesis2:      'Genesis Part 2',
    CrystalIsles:  'Crystal Isles',
    Ragnarok:      'Ragnarok',
    Valguero:      'Valguero',
    LostIsland:    'Lost Island',
    Fjordur:       'Fjordur',
  }
  return labels[map] ?? map
}

/** Trunca string com reticências */
export function truncate(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str
  return str.slice(0, maxLen - 1) + '…'
}

/** Valida se valor é uma porta TCP válida (1024–65535) */
export function isValidPort(value: string | number): boolean {
  const n = typeof value === 'string' ? parseInt(value, 10) : value
  return Number.isInteger(n) && n >= 1024 && n <= 65535
}

/** Extrai a mensagem de erro de qualquer tipo de thrown value */
export function errorMessage(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error) return err.message
  return String(err)
}

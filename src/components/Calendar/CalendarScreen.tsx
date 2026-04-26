import { useState, useEffect } from 'react'
import {
    format,
    addMonths,
    subMonths,
    startOfMonth,
    endOfMonth,
    startOfWeek,
    endOfWeek,
    isSameMonth,
    isSameDay,
    addDays,
    fromUnixTime,
    getUnixTime
} from 'date-fns'
import { motion } from 'framer-motion'
import {
    ChevronLeft,
    ChevronRight,
    Calendar as CalendarIcon,
    Plus,
    MapPin,
    Clock,
    ArrowLeft
} from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { cn } from '@/lib/utils'
import { useThemeStore } from '@/stores/themeStore'

interface CalendarEvent {
    id: string
    title: string
    description?: string
    location?: string
    start: number
    end: number
    is_all_day: boolean
    calendar_name: string
    color?: string
}

interface CalendarScreenProps {
    onBack: () => void
}

export function CalendarScreen({ onBack }: CalendarScreenProps) {
    const { accentColor } = useThemeStore()
    const [currentDate, setCurrentDate] = useState(new Date())
    const [selectedDate, setSelectedDate] = useState(new Date())
    const [events, setEvents] = useState<CalendarEvent[]>([])
    const [isLoading, setIsLoading] = useState(true)

    useEffect(() => {
        const fetchEvents = async () => {
            setIsLoading(true)
            try {
                const monthStart = startOfMonth(currentDate)
                const monthEnd = endOfMonth(currentDate)

                const results = await invoke<CalendarEvent[]>('list_calendar_events', {
                    start: getUnixTime(startOfWeek(monthStart)),
                    end: getUnixTime(endOfWeek(monthEnd))
                })
                setEvents(results)
            } catch (err) {
                console.error('Failed to fetch events:', err)
            } finally {
                setIsLoading(false)
            }
        }

        fetchEvents()
    }, [currentDate])

    const nextMonth = () => setCurrentDate(addMonths(currentDate, 1))
    const prevMonth = () => setCurrentDate(subMonths(currentDate, 1))

    const renderHeader = () => {
        return (
            <div className="flex items-center justify-between px-6 py-4">
                <div className="flex items-center gap-4">
                    <button
                        onClick={onBack}
                        className="p-2 rounded-full hover:bg-white/5 transition-colors text-muted-foreground hover:text-foreground"
                    >
                        <ArrowLeft className="w-5 h-5" />
                    </button>
                    <div className="flex flex-col">
                        <h2 className="text-xl font-bold tracking-tight">
                            {format(currentDate, 'MMMM yyyy')}
                        </h2>
                        <p className="text-xs text-muted-foreground uppercase tracking-widest font-medium">
                            {isLoading ? 'Syncing...' : 'OS Calendar'}
                        </p>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <div className="flex items-center bg-surface-panel/50 rounded-xl p-1 ring-1 ring-white/5">
                        <button
                            onClick={prevMonth}
                            className="p-2 rounded-lg hover:bg-white/5 transition-colors"
                        >
                            <ChevronLeft className="w-5 h-5" />
                        </button>
                        <button
                            onClick={() => setCurrentDate(new Date())}
                            className="px-4 py-2 text-sm font-medium rounded-lg hover:bg-white/5 transition-colors"
                        >
                            Today
                        </button>
                        <button
                            onClick={nextMonth}
                            className="p-2 rounded-lg hover:bg-white/5 transition-colors"
                        >
                            <ChevronRight className="w-5 h-5" />
                        </button>
                    </div>

                    <button className="flex items-center gap-2 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 transition-colors text-sm font-medium ring-1 ring-white/5">
                        <Plus className="w-4 h-4" />
                        Add Event
                    </button>
                </div>
            </div>
        )
    }

    const renderDays = () => {
        const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
        return (
            <div className="grid grid-cols-7 mb-2">
                {days.map((day) => (
                    <div key={day} className="text-center py-2">
                        <span className="text-xs font-bold text-muted-foreground uppercase tracking-widest">
                            {day}
                        </span>
                    </div>
                ))}
            </div>
        )
    }

    const renderCells = () => {
        const monthStart = startOfMonth(currentDate)
        const monthEnd = endOfMonth(monthStart)
        const startDate = startOfWeek(monthStart)
        const endDate = endOfWeek(monthEnd)

        const rows = []
        let days = []
        let day = startDate
        let formattedDate = ''

        while (day <= endDate) {
            for (let i = 0; i < 7; i++) {
                formattedDate = format(day, 'd')
                const cloneDay = day
                const dayEvents = events.filter(e => isSameDay(fromUnixTime(e.start), cloneDay))

                days.push(
                    <div
                        key={day.toString()}
                        className={cn(
                            "relative min-h-[120px] p-2 border-r border-b border-white/[0.03] transition-colors",
                            !isSameMonth(day, monthStart) ? "bg-black/20 text-white/20" : "hover:bg-white/[0.02]",
                            isSameDay(day, new Date()) && "bg-accent/5"
                        )}
                        onClick={() => setSelectedDate(cloneDay)}
                    >
                        <div className="flex items-center justify-between mb-1">
                            <span className={cn(
                                "text-sm font-medium w-7 h-7 flex items-center justify-center rounded-full transition-colors",
                                isSameDay(day, new Date()) ? "bg-accent text-white" : "",
                                isSameDay(day, selectedDate) && !isSameDay(day, new Date()) ? "ring-1 ring-accent text-accent" : ""
                            )}
                                style={isSameDay(day, new Date()) ? { backgroundColor: accentColor } : {}}
                            >
                                {formattedDate}
                            </span>
                        </div>

                        <div className="flex flex-col gap-1 overflow-hidden">
                            {dayEvents.slice(0, 3).map((event) => (
                                <div
                                    key={event.id}
                                    className="px-2 py-1 rounded-md text-[10px] font-medium truncate bg-white/5 border border-white/5 hover:bg-white/10 transition-colors"
                                >
                                    {event.title}
                                </div>
                            ))}
                            {dayEvents.length > 3 && (
                                <div className="text-[10px] text-muted-foreground px-2">
                                    + {dayEvents.length - 3} more
                                </div>
                            )}
                        </div>
                    </div>
                )
                day = addDays(day, 1)
            }
            rows.push(
                <div key={day.toString()} className="grid grid-cols-7 border-l border-white/[0.03]">
                    {days}
                </div>
            )
            days = []
        }
        return <div className="flex-1 overflow-y-auto">{rows}</div>
    }

    const renderSidebar = () => {
        const selectedEvents = events.filter(e => isSameDay(fromUnixTime(e.start), selectedDate))

        return (
            <div className="w-80 border-l border-white/[0.05] flex flex-col bg-surface-panel/30 backdrop-blur-md">
                <div className="p-6 border-b border-white/[0.05]">
                    <h3 className="text-lg font-bold">
                        {format(selectedDate, 'EEEE')}
                    </h3>
                    <p className="text-sm text-muted-foreground">
                        {format(selectedDate, 'MMMM do, yyyy')}
                    </p>
                </div>

                <div className="flex-1 overflow-y-auto p-4 flex flex-col gap-4">
                    {selectedEvents.length === 0 ? (
                        <div className="flex flex-col items-center justify-center py-12 text-center">
                            <div className="w-12 h-12 rounded-2xl bg-white/5 flex items-center justify-center mb-4 text-muted-foreground">
                                <CalendarIcon className="w-6 h-6" />
                            </div>
                            <p className="text-sm font-medium text-muted-foreground">No events scheduled</p>
                        </div>
                    ) : (
                        selectedEvents.map((event) => (
                            <motion.div
                                key={event.id}
                                initial={{ opacity: 0, x: 20 }}
                                animate={{ opacity: 1, x: 0 }}
                                className="p-4 rounded-2xl bg-white/[0.03] ring-1 ring-white/[0.05] flex flex-col gap-3 hover:bg-white/[0.05] transition-colors group cursor-default"
                            >
                                <div className="flex items-start justify-between">
                                    <h4 className="font-semibold text-sm leading-tight group-hover:text-accent transition-colors">
                                        {event.title}
                                    </h4>
                                    <div
                                        className="w-2 h-2 rounded-full mt-1"
                                        style={{ backgroundColor: event.color || accentColor }}
                                    />
                                </div>

                                {(event.description || event.location) && (
                                    <div className="flex flex-col gap-2">
                                        {event.location && (
                                            <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                                <MapPin className="w-3 h-3" />
                                                <span>{event.location}</span>
                                            </div>
                                        )}
                                        <div className="flex items-center gap-2 text-xs text-muted-foreground">
                                            <Clock className="w-3 h-3" />
                                            <span>
                                                {event.is_all_day ? 'All Day' : `${format(fromUnixTime(event.start), 'HH:mm')} - ${format(fromUnixTime(event.end), 'HH:mm')}`}
                                            </span>
                                        </div>
                                    </div>
                                )}

                                {event.description && (
                                    <p className="text-xs text-muted-foreground/80 line-clamp-2 italic">
                                        {event.description}
                                    </p>
                                )}
                            </motion.div>
                        ))
                    )}
                </div>
            </div>
        )
    }

    return (
        <div className="flex h-full flex-col bg-surface-panel/20">
            {renderHeader()}
            <div className="flex flex-1 overflow-hidden border-t border-white/[0.05]">
                <div className="flex-1 flex flex-col p-4 overflow-hidden">
                    {renderDays()}
                    <div className="flex-1 bg-surface-panel/40 rounded-2xl border border-white/[0.05] overflow-hidden flex flex-col shadow-2xl">
                        {renderCells()}
                    </div>
                </div>
                {renderSidebar()}
            </div>
        </div>
    )
}

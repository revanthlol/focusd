import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, Zap, ChevronLeft, ChevronRight } from "lucide-react";
import { BarChart, Bar, XAxis, Tooltip, ResponsiveContainer } from "recharts";
import { motion } from "framer-motion";

interface DashboardData {
  total_seconds: number;
  daily_average: number;
  comparison_seconds: number;     // positive = more, negative = less
  comparison_label: string;       // "vs yesterday" / "vs last week" / "vs last month"
  apps: [string, number][];
  daily_chart: [string, number][];
  hourly_chart: [number, number][]; // [hour, seconds] — today only
  heatmap: [string, number][];      // [date_str, seconds] — month only
  current_app: string | null;
  is_idle: boolean;
}

// 1. Time Formatter with Seconds
const fmt = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = Math.floor(s % 60); // Show seconds
  
  if (h > 0) return `${h}h ${m}m ${sec}s`;
  if (m > 0) return `${m}m ${sec}s`;
  return `${sec}s`;
};

// Simple clean tooltip
const CustomTooltip = ({ active, payload }: any) => {
  if (active && payload && payload.length) {
    return (
      <div className="bg-secondary text-secondary-foreground border border-border p-2 rounded text-xs">
        <span className="font-semibold">{payload[0].payload.name}</span>: {fmt(payload[0].value)}
      </div>
    );
  }
  return null;
};

function HeatmapGrid({ data, anchor }: { data: [string, number][]; anchor: string }) {
  const d = new Date(anchor);
  const year = d.getFullYear();
  const month = d.getMonth();
  
  // Build map of date -> seconds
  const map = new Map(data);
  const maxVal = Math.max(...data.map(([_, v]) => v), 1);
  
  // First day of month, day of week (0=Sun)
  const firstDay = new Date(year, month, 1).getDay();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  
  // Offset: Monday-start. Convert Sun=0 to 6, Mon=1 to 0, etc.
  const offset = firstDay === 0 ? 6 : firstDay - 1;
  
  const cells = [];
  // Empty cells for offset
  for (let i = 0; i < offset; i++) {
    cells.push(<div key={`empty-${i}`} className="w-4 h-4" />);
  }
  // Day cells
  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    const val = map.get(dateStr) || 0;
    const intensity = val / maxVal;
    const opacity = val === 0 ? 0.08 : 0.15 + intensity * 0.85;
    cells.push(
      <div
        key={day}
        title={`${dateStr}: ${fmt(val)}`}
        className="w-4 h-4 rounded-sm bg-primary transition-opacity"
        style={{ opacity }}
      />
    );
  }
  
  return (
    <div>
      {/* Day labels */}
      <div className="grid grid-cols-7 gap-1 mb-1">
        {["M","T","W","T","F","S","S"].map((l, i) => (
          <div key={i} className="w-4 h-4 text-[8px] text-muted-foreground text-center">{l}</div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1">
        {cells}
      </div>
    </div>
  );
}

export default function App() {
  const [view, setView] = useState<"today"|"week"|"month">("today");
  const [anchor, setAnchor] = useState<string>(new Date().toISOString().slice(0, 10));
  const [data, setData] = useState<DashboardData | null>(null);
  const [showAll, setShowAll] = useState(false);

  useEffect(() => {
    invoke<Record<string, string>>("get_theme")
      .then((theme: Record<string, string>) => {
        const root = document.documentElement;
        Object.entries(theme).forEach(([key, value]: [string, string]) => {
          root.style.setProperty(key, value);
        });
      })
      .catch(console.error);
  }, []); // empty deps = runs once on mount

  useEffect(() => {
    const handleFocus = () => {
      invoke<Record<string, string>>("get_theme")
        .then((theme: Record<string, string>) => {
          const root = document.documentElement;
          Object.entries(theme).forEach(([key, value]: [string, string]) => {
            root.style.setProperty(key, value);
          });
        })
        .catch(console.error);
    };
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, []);

  // Polling Logic
  useEffect(() => {
    const fetchData = () => {
      invoke<DashboardData>("get_dashboard", { view, anchor })
        .then(setData)
        .catch(console.error);
    };
    fetchData();
    // Adaptive polling: today=3s, week=15s, month=30s
    const ms = view === "today" ? 3000 : view === "week" ? 15000 : 30000;
    const int = setInterval(fetchData, ms);
    return () => clearInterval(int);
  }, [view, anchor]);

  const navigate = (dir: -1 | 1) => {
    const d = new Date(anchor);
    if (view === "today") d.setDate(d.getDate() + dir);
    else if (view === "week") d.setDate(d.getDate() + dir * 7);
    else d.setMonth(d.getMonth() + dir);
    setAnchor(d.toISOString().slice(0, 10));
  };

  const isToday = anchor === new Date().toISOString().slice(0, 10);

  const dateLabel = (() => {
    const d = new Date(anchor);
    if (view === "today") {
      return isToday ? "Today" : d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
    }
    if (view === "week") {
      const mon = new Date(d);
      mon.setDate(d.getDate() - d.getDay() + 1);
      const sun = new Date(mon);
      sun.setDate(mon.getDate() + 6);
      return `${mon.toLocaleDateString("en-US", { month: "short", day: "numeric" })} – ${sun.toLocaleDateString("en-US", { month: "short", day: "numeric" })}`;
    }
    return d.toLocaleDateString("en-US", { month: "long", year: "numeric" });
  })();

  // Determine Max for progress bars
  const maxVal = data?.apps.reduce((max, [_, v]) => Math.max(max, v), 0) || 1;

  return (
    <div className="dark min-h-screen bg-background text-foreground p-6 font-sans select-none flex flex-col">
      
      {/* HEADER */}
      <header className="flex justify-between items-end mb-6 pb-2">
        <div>
          <h1 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <Zap className="h-6 w-6 text-primary fill-primary" /> FOCUSD
          </h1>
          <p className="text-muted-foreground text-xs uppercase tracking-widest mt-1">Analytics</p>
        </div>
        
        {/* Status Indicator */}
        <div className="flex items-center gap-2 mb-1 bg-secondary/50 px-3 py-1.5 rounded-full border border-border/50">
          <span className="relative flex h-2 w-2">
            <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
              data?.is_idle ? "bg-muted-foreground" : "bg-primary"
            }`} />
            <span className={`relative inline-flex rounded-full h-2 w-2 ${
              data?.is_idle ? "bg-muted-foreground" : "bg-primary"
            }`} />
          </span>
          <span className="text-[10px] text-muted-foreground font-medium uppercase truncate max-w-[120px]">
            {data?.is_idle ? "Idle" : data?.current_app || "Live"}
          </span>
        </div>
      </header>

      {/* TABS */}
      <Tabs value={view} onValueChange={(v) => { setView(v as "today"|"week"|"month"); setAnchor(new Date().toISOString().slice(0,10)); setShowAll(false); }}>
        <TabsList className="grid w-full grid-cols-3 mb-4">
          <TabsTrigger value="today">Today</TabsTrigger>
          <TabsTrigger value="week">Week</TabsTrigger>
          <TabsTrigger value="month">Month</TabsTrigger>
        </TabsList>
      </Tabs>

      {/* DATE NAVIGATION */}
      <div className="flex items-center justify-center gap-4 mb-6">
        <button onClick={() => navigate(-1)} className="text-muted-foreground hover:text-foreground transition p-1">
          <ChevronLeft className="h-5 w-5" />
        </button>
        <span className="text-sm font-medium min-w-[160px] text-center">{dateLabel}</span>
        <button onClick={() => navigate(1)} disabled={isToday} className="text-muted-foreground hover:text-foreground transition p-1 disabled:opacity-30">
          <ChevronRight className="h-5 w-5" />
        </button>
      </div>

      {/* SUMMARY CARDS ROW */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        {/* Card 1: Daily Average (only week/month) or Total (today) */}
        <Card className="bg-secondary/20 border-border/50">
          <CardContent className="pt-4 pb-4 px-4">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
              {view === "today" ? "Total Time" : "Daily Average"}
            </p>
            <p className="text-2xl font-bold tabular-nums">
              {data ? fmt(view === "today" ? data.total_seconds : data.daily_average) : <span className="inline-block h-7 w-20 bg-secondary rounded animate-pulse" />}
            </p>
          </CardContent>
        </Card>

        {/* Card 2: Total Time (big, center emphasis) */}
        <Card className="bg-secondary/20 border-border/50">
          <CardContent className="pt-4 pb-4 px-4 text-center">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
              {view === "today" ? "Today" : view === "week" ? "This Week" : "This Month"}
            </p>
            <p className="text-3xl font-extrabold tabular-nums">
              {data ? fmt(data.total_seconds) : <span className="inline-block h-8 w-24 bg-secondary rounded animate-pulse" />}
            </p>
          </CardContent>
        </Card>

        {/* Card 3: Comparison */}
        <Card className="bg-secondary/20 border-border/50">
          <CardContent className="pt-4 pb-4 px-4 text-right">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-1">
              {data?.comparison_label || "—"}
            </p>
            <p className={`text-2xl font-bold tabular-nums ${
              data && data.comparison_seconds > 0 ? "text-green-400" 
              : data && data.comparison_seconds < 0 ? "text-red-400" 
              : "text-muted-foreground"
            }`}>
              {data ? `${data.comparison_seconds >= 0 ? "↑" : "↓"} ${fmt(Math.abs(data.comparison_seconds))}` : <span className="inline-block h-7 w-20 bg-secondary rounded animate-pulse" />}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* MAIN CONTENT AREA */}
      <div className="flex-1 grid grid-cols-1 md:grid-cols-2 gap-6 overflow-hidden">
        
        {/* LEFT: Charts */}
        <div className="space-y-6 flex flex-col">
          
          {/* TODAY: Hourly chart */}
          {view === "today" && data && data.hourly_chart.length > 0 && (
            <Card className="flex-1 border-border/50">
              <CardHeader className="pb-2">
                <CardTitle className="text-xs uppercase text-muted-foreground tracking-wider">Activity Timeline</CardTitle>
              </CardHeader>
              <CardContent className="p-4 h-full min-h-[200px]">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={data.hourly_chart.map(([h, v]) => ({ name: `${h}:00`, value: v }))}>
                    <XAxis dataKey="name" stroke="hsl(var(--muted-foreground))" fontSize={10} tickLine={false} axisLine={false} interval={2} />
                    <Tooltip cursor={{fill: 'hsl(var(--secondary))'}} content={<CustomTooltip />} />
                    <Bar dataKey="value" radius={[2, 2, 0, 0]} className="fill-primary" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          )}

          {/* WEEK: Daily bar chart */}
          {view === "week" && (
            <Card className="flex-1 border-border/50">
              <CardHeader className="pb-2">
                <CardTitle className="text-xs uppercase text-muted-foreground tracking-wider">Daily Breakdown</CardTitle>
              </CardHeader>
              <CardContent className="p-4 h-full min-h-[200px]">
                <ResponsiveContainer width="100%" height="100%">
                  <BarChart data={data?.daily_chart.map(([name, v]) => ({ name, value: v })) || []}>
                    <XAxis dataKey="name" stroke="hsl(var(--muted-foreground))" fontSize={12} tickLine={false} axisLine={false} />
                    <Tooltip cursor={{fill: 'hsl(var(--secondary))'}} content={<CustomTooltip />} />
                    <Bar dataKey="value" radius={[4, 4, 0, 0]} className="fill-primary" />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
          )}

          {/* MONTH: Heatmap calendar */}
          {view === "month" && data && (
            <Card className="flex-1 border-border/50">
              <CardHeader className="pb-2">
                <CardTitle className="text-xs uppercase text-muted-foreground tracking-wider">Activity Heatmap</CardTitle>
              </CardHeader>
              <CardContent className="p-4">
                <HeatmapGrid data={data.heatmap} anchor={anchor} />
              </CardContent>
            </Card>
          )}

          {/* Loading state for today */}
          {view === "today" && !data && (
            <Card className="flex-1 border-border/50">
              <CardContent className="p-4 h-full min-h-[200px] flex items-center justify-center">
                <div className="text-muted-foreground text-sm animate-pulse">Loading...</div>
              </CardContent>
            </Card>
          )}

          {/* Empty state for today with no hourly data */}
          {view === "today" && data && data.hourly_chart.length === 0 && (
            <Card className="flex-1 border-border/50">
              <CardContent className="p-4 h-full min-h-[200px] flex items-center justify-center">
                <p className="text-muted-foreground text-sm">No activity recorded yet</p>
              </CardContent>
            </Card>
          )}
        </div>

        {/* RIGHT: App list */}
        <Card className="border-border/50 flex flex-col overflow-hidden max-h-[calc(100vh-420px)] min-h-[200px]">
          <CardHeader className="bg-secondary/30 pb-3 border-b border-border/50 flex flex-row items-center justify-between">
            <CardTitle className="text-xs font-medium uppercase flex items-center gap-2 text-muted-foreground tracking-wider">
              <Activity className="h-4 w-4" /> Top Applications
            </CardTitle>
            {data && data.apps.length > 8 && (
              <button
                onClick={() => setShowAll(!showAll)}
                className="text-[10px] text-muted-foreground hover:text-foreground transition uppercase tracking-wider"
              >
                {showAll ? "Show Less" : `Show All (${data.apps.length})`}
              </button>
            )}
          </CardHeader>
          <CardContent className="p-0 flex-1 relative">
            <ScrollArea className="h-full w-full">
              <div className="p-4 space-y-3">
                {(showAll ? data?.apps : data?.apps.slice(0, 8))?.map(([name, sec], i) => {
                  const pct = data ? Math.round((sec / data.total_seconds) * 100) : 0;
                  const barPct = (sec / maxVal) * 100;
                  return (
                    <div key={name} className="group">
                      <div className="flex justify-between items-center mb-1">
                        <div className="flex items-center gap-2 min-w-0">
                          <span className="text-sm font-medium truncate group-hover:text-primary transition-colors">
                            {name}
                          </span>
                          <span className="text-[10px] text-muted-foreground tabular-nums">
                            {pct}%
                          </span>
                        </div>
                        <span className="text-xs text-muted-foreground font-mono tabular-nums ml-2 flex-shrink-0">
                          {fmt(sec)}
                        </span>
                      </div>
                      <div className="h-1.5 w-full bg-secondary rounded-full overflow-hidden">
                        <motion.div
                          initial={{ width: 0 }}
                          animate={{ width: `${barPct}%` }}
                          transition={{ duration: 0.4, delay: i * 0.03 }}
                          className="h-full bg-primary rounded-full"
                        />
                      </div>
                    </div>
                  );
                })}

                {/* Loading skeleton */}
                {!data && (
                  <div className="space-y-4 p-2">
                    {[...Array(5)].map((_, i) => (
                      <div key={i} className="space-y-2 animate-pulse">
                        <div className="flex justify-between">
                          <div className="h-3 bg-secondary rounded w-24" />
                          <div className="h-3 bg-secondary rounded w-12" />
                        </div>
                        <div className="h-1.5 bg-secondary rounded-full" />
                      </div>
                    ))}
                  </div>
                )}

                {/* Empty state */}
                {data && data.apps.length === 0 && (
                  <div className="py-8 text-center text-muted-foreground text-sm">
                    No activity recorded
                  </div>
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
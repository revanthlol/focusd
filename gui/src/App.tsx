import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Activity, Zap, ChevronLeft, ChevronRight } from "lucide-react";
import { BarChart, Bar, XAxis, Tooltip, ResponsiveContainer } from "recharts";
import { motion } from "framer-motion";

interface DashboardData {
  total_seconds: number;
  daily_average: number;
  comparison_seconds: number;
  comparison_label: string;
  apps: [string, number][];
  daily_chart: [string, number][];
  hourly_chart: [number, number][];
  heatmap: [string, number][];
  current_app: string | null;
  is_idle: boolean;
}

const fmt = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = Math.floor(s % 60);
  if (h > 0) return `${h}h ${m}m ${sec}s`;
  if (m > 0) return `${m}m ${sec}s`;
  return `${sec}s`;
};

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

function Skeleton() {
  return <span className="inline-block h-8 w-24 bg-secondary rounded animate-pulse align-middle" />;
}

function HeatmapGrid({ data, anchor }: { data: [string, number][]; anchor: string }) {
  const d = new Date(anchor);
  const year = d.getFullYear();
  const month = d.getMonth();

  const map = new Map(data);
  const maxVal = Math.max(...data.map(([_, v]) => v), 1);

  const firstDay = new Date(year, month, 1).getDay();
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const offset = firstDay === 0 ? 6 : firstDay - 1;

  const cells: React.ReactNode[] = [];
  for (let i = 0; i < offset; i++) {
    cells.push(<div key={`empty-${i}`} className="w-5 h-5" />);
  }
  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${year}-${String(month + 1).padStart(2, "0")}-${String(day).padStart(2, "0")}`;
    const val = map.get(dateStr) || 0;
    const intensity = val / maxVal;
    const opacity = val === 0 ? 0.08 : 0.15 + intensity * 0.85;
    cells.push(
      <div
        key={day}
        title={`${dateStr}: ${fmt(val)}`}
        className="w-5 h-5 rounded-sm bg-primary transition-opacity"
        style={{ opacity }}
      />
    );
  }

  return (
    <div className="flex flex-col items-center">
      <div className="grid grid-cols-7 gap-1.5 mb-1.5">
        {["M", "T", "W", "T", "F", "S", "S"].map((l, i) => (
          <div key={i} className="w-5 h-5 text-[9px] text-muted-foreground text-center leading-5">{l}</div>
        ))}
      </div>
      <div className="grid grid-cols-7 gap-1.5">
        {cells}
      </div>
    </div>
  );
}

export default function App() {
  const [view, setView] = useState<"today" | "week" | "month">("today");
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
  }, []);

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

  useEffect(() => {
    const fetchData = () => {
      invoke<DashboardData>("get_dashboard", { view, anchor })
        .then(setData)
        .catch(console.error);
    };
    fetchData();
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

  const maxVal = data?.apps.reduce((max, [_, v]) => Math.max(max, v), 0) || 1;

  return (
    <div className="dark min-h-screen bg-background text-foreground select-none">
      <div className="max-w-[520px] mx-auto px-5 py-6 flex flex-col min-h-screen">

        {/* HEADER */}
        <header className="flex justify-between items-center mb-5">
          <div>
            <h1 className="text-xl font-bold tracking-tight flex items-center gap-2">
              <Zap className="h-5 w-5 text-primary fill-primary" /> FOCUSD
            </h1>
            <p className="text-muted-foreground text-[10px] uppercase tracking-[0.2em] mt-0.5">Analytics</p>
          </div>

          {/* Active Now Badge */}
          <div className="flex items-center gap-2 bg-secondary/60 px-3 py-1.5 rounded-full border border-border/40">
            <span className="relative flex h-2 w-2">
              <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
                data?.is_idle ? "bg-muted-foreground" : "bg-primary"
              }`} />
              <span className={`relative inline-flex rounded-full h-2 w-2 ${
                data?.is_idle ? "bg-muted-foreground" : "bg-primary"
              }`} />
            </span>
            <span className="text-[10px] text-muted-foreground font-medium uppercase truncate max-w-[100px]">
              {data?.is_idle ? "Idle" : data?.current_app || "Live"}
            </span>
          </div>
        </header>

        {/* TABS */}
        <Tabs value={view} onValueChange={(v) => { setView(v as "today" | "week" | "month"); setAnchor(new Date().toISOString().slice(0, 10)); setShowAll(false); }}>
          <TabsList className="grid w-full grid-cols-3 mb-4 h-10">
            <TabsTrigger value="today" className="text-sm">Today</TabsTrigger>
            <TabsTrigger value="week" className="text-sm">Week</TabsTrigger>
            <TabsTrigger value="month" className="text-sm">Month</TabsTrigger>
          </TabsList>
        </Tabs>

        {/* DATE NAV */}
        <div className="flex items-center justify-center gap-6 mb-5">
          <button onClick={() => navigate(-1)} className="text-muted-foreground hover:text-foreground transition-colors p-1.5 rounded-lg hover:bg-secondary/50">
            <ChevronLeft className="h-4 w-4" />
          </button>
          <span className="text-sm font-semibold min-w-[140px] text-center">{dateLabel}</span>
          <button onClick={() => navigate(1)} disabled={isToday} className="text-muted-foreground hover:text-foreground transition-colors p-1.5 rounded-lg hover:bg-secondary/50 disabled:opacity-20 disabled:hover:bg-transparent">
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>

        {/* SCROLLABLE CONTENT */}
        <div className="flex-1 space-y-4 pb-6">

          {/* SUMMARY CARDS */}
          {view === "today" ? (
            <>
              <Card className="bg-secondary/20 border-border/40">
                <CardContent className="py-5 px-5 text-center">
                  <p className="text-[10px] text-muted-foreground uppercase tracking-[0.15em] mb-2">Total Time Today</p>
                  <p className="text-5xl font-extrabold tabular-nums tracking-tight">
                    {data ? fmt(data.total_seconds) : <Skeleton />}
                  </p>
                </CardContent>
              </Card>
              <Card className="bg-secondary/20 border-border/40">
                <CardContent className="py-4 px-5 text-right">
                  <p className="text-[10px] text-muted-foreground uppercase tracking-[0.15em] mb-1">
                    {data?.comparison_label || "—"}
                  </p>
                  <p className={`text-2xl font-bold tabular-nums ${
                    data && data.comparison_seconds > 0 ? "text-green-400"
                    : data && data.comparison_seconds < 0 ? "text-red-400"
                    : "text-muted-foreground"
                  }`}>
                    {data
                      ? `${data.comparison_seconds >= 0 ? "↑" : "↓"} ${fmt(Math.abs(data.comparison_seconds))}`
                      : <Skeleton />}
                  </p>
                </CardContent>
              </Card>
            </>
          ) : (
            <>
              <div className="grid grid-cols-2 gap-4">
                <Card className="bg-secondary/20 border-border/40">
                  <CardContent className="py-4 px-4">
                    <p className="text-[10px] text-muted-foreground uppercase tracking-[0.15em] mb-1">Daily Average</p>
                    <p className="text-2xl font-bold tabular-nums">
                      {data ? fmt(data.daily_average) : <Skeleton />}
                    </p>
                  </CardContent>
                </Card>
                <Card className="bg-secondary/20 border-border/40">
                  <CardContent className="py-4 px-4 text-right">
                    <p className="text-[10px] text-muted-foreground uppercase tracking-[0.15em] mb-1">
                      {data?.comparison_label || "—"}
                    </p>
                    <p className={`text-2xl font-bold tabular-nums ${
                      data && data.comparison_seconds > 0 ? "text-green-400"
                      : data && data.comparison_seconds < 0 ? "text-red-400"
                      : "text-muted-foreground"
                    }`}>
                      {data
                        ? `${data.comparison_seconds >= 0 ? "↑" : "↓"} ${fmt(Math.abs(data.comparison_seconds))}`
                        : <Skeleton />}
                    </p>
                  </CardContent>
                </Card>
              </div>
              <Card className="bg-secondary/20 border-border/40">
                <CardContent className="py-4 px-5 text-center">
                  <p className="text-[10px] text-muted-foreground uppercase tracking-[0.15em] mb-1">
                    {view === "week" ? "This Week" : "This Month"}
                  </p>
                  <p className="text-4xl font-extrabold tabular-nums tracking-tight">
                    {data ? fmt(data.total_seconds) : <Skeleton />}
                  </p>
                </CardContent>
              </Card>
            </>
          )}

          {/* TODAY: Hourly chart */}
          {view === "today" && data && data.hourly_chart.length > 0 && (
            <Card className="border-border/40">
              <CardHeader className="pb-1 pt-4 px-5">
                <CardTitle className="text-[10px] uppercase text-muted-foreground tracking-[0.15em]">Activity Timeline</CardTitle>
              </CardHeader>
              <CardContent className="px-3 pb-4">
                <div className="h-[160px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={data.hourly_chart.map(([h, v]) => ({ name: `${h}`, value: v }))}>
                      <XAxis dataKey="name" stroke="hsl(var(--muted-foreground))" fontSize={9} tickLine={false} axisLine={false} interval={3} />
                      <Tooltip cursor={{ fill: 'hsl(var(--secondary) / 0.5)' }} content={<CustomTooltip />} />
                      <Bar dataKey="value" radius={[3, 3, 0, 0]} className="fill-primary" animationDuration={600} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          )}

          {/* WEEK: Daily bar chart */}
          {view === "week" && (
            <Card className="border-border/40">
              <CardHeader className="pb-1 pt-4 px-5">
                <CardTitle className="text-[10px] uppercase text-muted-foreground tracking-[0.15em]">Daily Breakdown</CardTitle>
              </CardHeader>
              <CardContent className="px-3 pb-4">
                <div className="h-[180px]">
                  <ResponsiveContainer width="100%" height="100%">
                    <BarChart data={data?.daily_chart.map(([name, v]) => ({ name, value: v })) || []}>
                      <XAxis dataKey="name" stroke="hsl(var(--muted-foreground))" fontSize={11} tickLine={false} axisLine={false} />
                      <Tooltip cursor={{ fill: 'hsl(var(--secondary) / 0.5)' }} content={<CustomTooltip />} />
                      <Bar dataKey="value" radius={[4, 4, 0, 0]} className="fill-primary" animationDuration={600} />
                    </BarChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          )}

          {/* MONTH: Heatmap */}
          {view === "month" && data && (
            <Card className="border-border/40">
              <CardHeader className="pb-2 pt-4 px-5">
                <CardTitle className="text-[10px] uppercase text-muted-foreground tracking-[0.15em]">Activity Heatmap</CardTitle>
              </CardHeader>
              <CardContent className="px-5 pb-4">
                <HeatmapGrid data={data.heatmap} anchor={anchor} />
              </CardContent>
            </Card>
          )}

          {/* Loading fallback */}
          {view === "today" && !data && (
            <Card className="border-border/40">
              <CardContent className="py-12 flex items-center justify-center">
                <div className="text-muted-foreground text-sm animate-pulse">Loading...</div>
              </CardContent>
            </Card>
          )}

          {/* Empty fallback */}
          {view === "today" && data && data.hourly_chart.length === 0 && (
            <Card className="border-border/40">
              <CardContent className="py-12 flex items-center justify-center">
                <p className="text-muted-foreground text-sm">No activity recorded yet</p>
              </CardContent>
            </Card>
          )}

          {/* APP LIST */}
          <Card className="border-border/40">
            <CardHeader className="pb-2 pt-4 px-5 flex flex-row items-center justify-between">
              <CardTitle className="text-[10px] font-medium uppercase flex items-center gap-2 text-muted-foreground tracking-[0.15em]">
                <Activity className="h-3.5 w-3.5" /> Top Applications
              </CardTitle>
              {data && data.apps.length > 8 && (
                <button
                  onClick={() => setShowAll(!showAll)}
                  className="text-[10px] text-muted-foreground hover:text-foreground transition-colors uppercase tracking-wider"
                >
                  {showAll ? "Less" : `All (${data.apps.length})`}
                </button>
              )}
            </CardHeader>
            <CardContent className="px-5 pb-4">
              <div className="space-y-4">
                {(showAll ? data?.apps : data?.apps.slice(0, 8))?.map(([name, sec], i) => {
                  const pct = data ? Math.round((sec / data.total_seconds) * 100) : 0;
                  const barPct = (sec / maxVal) * 100;
                  const hue = name.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 360;
                  return (
                    <div key={name} className="group">
                      <div className="flex items-center gap-3 mb-1.5">
                        {/* App icon avatar */}
                        <div
                          className="w-7 h-7 rounded-lg flex items-center justify-center text-[11px] font-bold flex-shrink-0"
                          style={{ backgroundColor: `hsl(${hue} 50% 25%)`, color: `hsl(${hue} 60% 75%)` }}
                        >
                          {name.charAt(0).toUpperCase()}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex justify-between items-baseline">
                            <span className="text-sm font-semibold truncate group-hover:text-primary transition-colors">
                              {name}
                            </span>
                            <span className="text-xs text-muted-foreground font-mono tabular-nums ml-2 flex-shrink-0">
                              {fmt(sec)}
                            </span>
                          </div>
                          <div className="flex items-center gap-2 mt-1">
                            <div className="h-[5px] flex-1 bg-secondary rounded-full overflow-hidden">
                              <motion.div
                                initial={{ width: 0 }}
                                animate={{ width: `${barPct}%` }}
                                transition={{ type: "spring", stiffness: 100, damping: 20, delay: i * 0.04 }}
                                className="h-full bg-primary rounded-full"
                              />
                            </div>
                            <span className="text-[10px] text-muted-foreground tabular-nums w-7 text-right">
                              {pct}%
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })}

                {/* Skeleton loading */}
                {!data && (
                  <div className="space-y-5">
                    {[...Array(5)].map((_, i) => (
                      <div key={i} className="flex items-center gap-3 animate-pulse">
                        <div className="w-7 h-7 rounded-lg bg-secondary" />
                        <div className="flex-1 space-y-2">
                          <div className="flex justify-between">
                            <div className="h-3 bg-secondary rounded w-20" />
                            <div className="h-3 bg-secondary rounded w-12" />
                          </div>
                          <div className="h-[5px] bg-secondary rounded-full" />
                        </div>
                      </div>
                    ))}
                  </div>
                )}

                {data && data.apps.length === 0 && (
                  <div className="py-6 text-center text-muted-foreground text-sm">
                    No activity recorded
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

        </div>
      </div>
    </div>
  );
}
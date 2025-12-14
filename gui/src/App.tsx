import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Activity, Clock, Zap } from "lucide-react";
import { BarChart, Bar, XAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import { motion, AnimatePresence } from "framer-motion";

interface Data { 
  total_seconds: number; 
  apps: [string, number][]; 
  chart: [string, number][];
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

export default function App() {
  const [view, setView] = useState("today");
  const [data, setData] = useState<Data|null>(null);

  // Polling Logic
  useEffect(() => {
    const fetch = () => invoke<Data>("get_data", { view }).then(setData).catch(console.error);
    fetch();
    const int = setInterval(fetch, 1000); // 1s refresh for seconds accuracy
    return () => clearInterval(int);
  }, [view]);

  // Determine Max for progress bars
  const maxVal = data?.apps.reduce((max, [_, v]) => Math.max(max, v), 0) || 1;

  return (
    <div className="dark min-h-screen bg-background text-foreground p-6 font-sans select-none flex flex-col overflow-hidden">
      
      {/* HEADER */}
      <header className="flex justify-between items-end mb-6 pb-2">
        <div>
          <h1 className="text-2xl font-bold tracking-tight flex items-center gap-2">
            <Zap className="h-6 w-6 text-white fill-white" /> FOCUSD
          </h1>
          <p className="text-muted-foreground text-xs uppercase tracking-widest mt-1">Analytics</p>
        </div>
        
        {/* Status Indicator */}
        <div className="flex items-center gap-2 mb-1 bg-secondary/50 px-3 py-1 rounded-full border border-border/50">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-white"></span>
          </span>
          <span className="text-[10px] text-muted-foreground font-medium uppercase">Live</span>
        </div>
      </header>

      {/* TABS CONTAINER */}
      <Tabs defaultValue="today" onValueChange={setView} className="flex-1 flex flex-col h-full overflow-hidden">
        
        {/* 2. FIXED TABS SIZE (Full Width) */}
        <TabsList className="grid w-full grid-cols-2 mb-6">
          <TabsTrigger value="today">Today</TabsTrigger>
          <TabsTrigger value="week">This Week</TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-hidden">
          <AnimatePresence mode="wait">
            
            <motion.div
              key={view}
              initial={{ opacity: 0, x: 10 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -10 }}
              transition={{ duration: 0.2 }}
              className="grid grid-cols-1 md:grid-cols-2 gap-6 h-full"
            >
              
              {/* LEFT COL: Overview */}
              <div className="space-y-6 flex flex-col">
                <Card className="shadow-lg border-border/50 bg-secondary/20">
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
                      Total Time
                    </CardTitle>
                    <Clock className="h-4 w-4 text-muted-foreground" />
                  </CardHeader>
                  <CardContent>
                    {/* Mono font for seconds to prevent layout jitter */}
                    <div className="text-5xl font-extrabold tracking-tighter tabular-nums">
                      {data ? fmt(data.total_seconds) : "--"}
                    </div>
                  </CardContent>
                </Card>

                {/* WEEKLY CHART (Conditional) */}
                {view === "week" && (
                  <Card className="flex-1 border-border/50">
                    <CardContent className="p-4 h-full min-h-[200px]">
                      <ResponsiveContainer width="100%" height="100%">
                        <BarChart data={data?.chart.map(([name, v]) => ({ name, value: v })) || []}>
                          <XAxis 
                            dataKey="name" 
                            stroke="#888888" 
                            fontSize={12} 
                            tickLine={false} 
                            axisLine={false} 
                          />
                          <Tooltip cursor={{fill: 'var(--secondary)'}} content={<CustomTooltip />} />
                          <Bar dataKey="value" radius={[4, 4, 0, 0]} className="fill-primary" />
                        </BarChart>
                      </ResponsiveContainer>
                    </CardContent>
                  </Card>
                )}
                
                {/* Placeholder for Today view if you want something else there */}
                {view === "today" && (
                   <div className="flex-1 rounded-xl bg-secondary/20 border border-border/50 flex items-center justify-center text-muted-foreground text-sm">
                      <div className="text-center p-6">
                        <p>"Focus is the key to all success."</p>
                      </div>
                   </div>
                )}
              </div>

              {/* RIGHT COL: List */}
              <Card className="border-border/50 flex flex-col overflow-hidden h-[calc(100vh-180px)]">
                <CardHeader className="bg-secondary/30 pb-3 border-b border-border/50">
                  <CardTitle className="text-sm font-medium uppercase flex items-center gap-2 text-muted-foreground">
                    <Activity className="h-4 w-4" /> Activity Log
                  </CardTitle>
                </CardHeader>
                <CardContent className="p-0 flex-1 relative">
                  <ScrollArea className="h-full w-full p-4">
                    <div className="space-y-4">
                      {data?.apps.map(([name, sec], i) => {
                        const pct = (sec / maxVal) * 100;
                        return (
                          <div key={name} className="space-y-1.5 group">
                            <div className="flex justify-between items-center text-sm">
                              <span className="font-medium group-hover:text-primary transition-colors">{name}</span>
                              <span className="text-muted-foreground text-xs font-mono">{fmt(sec)}</span>
                            </div>
                            <div className="h-1.5 w-full bg-secondary rounded-full overflow-hidden">
                              <motion.div 
                                initial={{ width: 0 }}
                                animate={{ width: `${pct}%` }}
                                transition={{ duration: 0.5, delay: i * 0.05 }}
                                className="h-full bg-primary rounded-full"
                              />
                            </div>
                          </div>
                        )
                      })}
                      {!data && <div className="p-10 text-center text-muted-foreground">Connecting...</div>}
                    </div>
                  </ScrollArea>
                </CardContent>
              </Card>

            </motion.div>
          </AnimatePresence>
        </div>
      </Tabs>
    </div>
  );
}
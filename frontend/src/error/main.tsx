import { invoke } from "@tauri-apps/api/core";
import { WifiOff } from "lucide-react";
import { Button } from "../components/ui/button";
import { mount } from "../lib/boot";

mount(
  <main role="alert" className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
    <WifiOff size={36} strokeWidth={1.5} className="text-secondary" aria-hidden="true" />
    <p className="text-[15px] font-medium">Cannot reach WhatsApp. Check your connection.</p>
    <Button variant="primary" onClick={() => invoke("retry_account")}>
      Retry
    </Button>
  </main>,
);

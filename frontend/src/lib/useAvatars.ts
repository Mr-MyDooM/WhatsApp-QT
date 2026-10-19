import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** Account id to profile picture data URL. An account without a picture is absent. */
export function useAvatars(): Record<string, string> {
  const [avatars, setAvatars] = useState<Record<string, string>>({});
  useEffect(() => {
    let stopped = false;
    const stop = listen<Record<string, string>>("avatars", (e) => setAvatars(e.payload));
    invoke<Record<string, string>>("get_avatars").then((a) => {
      if (!stopped) setAvatars(a);
    });
    return () => {
      stopped = true;
      stop.then((f) => f());
    };
  }, []);
  return avatars;
}

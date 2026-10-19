import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { mount } from "../lib/boot";

const MAX_NAME_LENGTH = 40;
const params = new URLSearchParams(location.search);

function Rename() {
  const [name, setName] = useState(params.get("name") ?? "");
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    input.current?.focus();
    input.current?.select();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") invoke("cancel_rename");
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  function submit(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    invoke("rename_account", { id: params.get("id"), name });
  }

  return (
    <form onSubmit={submit} className="flex h-full flex-col justify-between p-5">
      <div className="flex flex-col gap-1.5">
        <label htmlFor="name" className="text-[13px] font-medium">
          Account name
        </label>
        <Input
          id="name"
          ref={input}
          value={name}
          maxLength={MAX_NAME_LENGTH}
          required
          onChange={(e) => setName(e.target.value)}
        />
      </div>
      <div className="flex justify-end gap-2">
        <Button onClick={() => invoke("cancel_rename")}>Cancel</Button>
        <Button type="submit" variant="primary">
          Rename
        </Button>
      </div>
    </form>
  );
}

mount(<Rename />);

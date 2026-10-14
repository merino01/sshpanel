import { describe, expect, it } from "vitest";
import { parseProfileForm, type ProfileForm } from "./profileForm";

const form: ProfileForm = { host: " vps.ejemplo.com ", port: "22", username: "alice", mode: "simple", sharedFolder: "", admin: false };

describe("parseProfileForm", () => {
  it("limpia los espacios y deja la Carpeta compartida vacía como ninguna", () => {
    expect(parseProfileForm(form)).toEqual({
      profile: { host: "vps.ejemplo.com", port: 22, username: "alice", mode: "simple", shared_folder: null, admin: false },
    });
    expect(parseProfileForm({ ...form, sharedFolder: " /srv/compartida ", admin: true, mode: "advanced" })).toMatchObject({
      profile: { shared_folder: "/srv/compartida", admin: true, mode: "advanced" },
    });
  });

  it("dice qué campos están mal", () => {
    expect(parseProfileForm({ ...form, host: "", port: "0", username: "Alice", sharedFolder: "compartida" })).toEqual({
      invalid: ["host", "port", "username", "sharedFolder"],
    });
    expect(parseProfileForm({ ...form, host: "vps.ejemplo.com/x", port: "70000", username: "1alice" })).toEqual({
      invalid: ["host", "port", "username"],
    });
    expect(parseProfileForm({ ...form, port: "22a" })).toEqual({ invalid: ["port"] });
  });
});

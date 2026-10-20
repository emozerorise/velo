export type WorkspaceView = "summary" | "transcript";
export type WorkspaceMode = "reading" | "video";
export type WorkspaceTool = "transcript" | "chat" | "player";

export interface VideoViewportInput {
  rect: { x: number; y: number; width: number; height: number };
  css_width: number;
  css_height: number;
  visible: boolean;
  revision: number;
}

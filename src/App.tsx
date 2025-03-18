import { useEffect, useState } from "react";
import {
  archiveListFiles,
  archiveExtract,
  ArchiveEntry,
  TreeNode,
} from "./commands";

import FileTree from "./components/FileTree";
import PasswordDialog from "./components/PasswordDialog";
import { run_args } from "./commands";
import "./App.css";

import { getCurrent } from "@tauri-apps/plugin-deep-link";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";

import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@heroui/react";

function App() {
  const [v, setV] = useState("");
  const [ww, setWW] = useState(800);
  const [hh, setHH] = useState(600);
  const [uzLoading, setUzloading] = useState(false);
  const [passwordRequire, setPasswordRequire] = useState(false);
  const [password, setPassword] = useState("");

  const [zipList, setZipList] = useState<TreeNode<ArchiveEntry>[]>([]);

  const _extensions = [
    "7z",
    "rar",
    "zip",
    "tar",
    "tar.gz",
    "tar.xz",
    "tar.bz2",
    "tbz2",
    "tbz",
  ];

  const setValue = (value: string) => {
    setV(decodeURIComponent(value));
  };

  useEffect(() => {
    getCurrentWindow()
      .innerSize()
      .then(async (res) => {
        const scaleFactor = await getCurrentWindow().scaleFactor();
        console.log("res :", res, res.width / scaleFactor);
        const width = res.width / scaleFactor - 40;
        const height = res.height / scaleFactor - 40;

        setWW(width || 780);
        setHH(height || 600);
      });
    getCurrentWindow().onResized(async (res) => {
      const scaleFactor = await getCurrentWindow().scaleFactor();
      const width = res.payload.width / scaleFactor - 40;
      const height = res.payload.height / scaleFactor - 40;
      setWW(width || 780);
      setHH(height || 600);
    });
  }, []);
  const win_run_args = async () => {
    try {
      const path = await run_args();
      if (path) {
        setValue(path);
      }
    } catch (err) {
      console.log("win error :", err);
    }
  };

  const macos_run_args = async () => {
    try {
      const path = (await getCurrent())?.[0];
      if (path && path.startsWith("file:///")) {
        setValue(path.replace("file://", ""));
        loadList(v, password);
      }
    } catch (err) {
      console.log("macos error :", err);
    }
  };

  const listenFileDrop = async () => {
    return getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const [path] = event.payload.paths;
        if (path) {
          loadList(path, password);
        }
      }
    });
  };
  const _un_zip_contents = async () => {
    const file = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "zip",
          extensions: ["zip"],
        },
        {
          name: "rar",
          extensions: ["rar"],
        },
        {
          name: "tar",
          extensions: [
            "tar",
            "tar.gz",
            "tar.xz",
            "tar.gz",
            "tar.bz2",
            "tbz2",
            "tbz",
          ],
        },
        {
          name: "7z",
          extensions: ["7z"],
        },
      ],
    });

    if (file) {
      loadList(file, password);
    }
  };

  const loadList = (filePath: string, password: string) => {
    setValue(filePath);
    archiveListFiles(filePath, password)
      .then((res) => {
        setZipList([...res]);
        console.log(JSON.stringify(res, null, 2));
      })
      .catch((err) => {
        if (err.MsgError == "密码错误") {
          setPasswordRequire(true);
        } else {
          alert(JSON.stringify(err));
          console.log("err :", err);
        }
      });
  };

  const handlePasswordConfirm = (password: string) => {
    loadList(v, password);
    setPassword(password);
    setPasswordRequire(false);
  };

  const zip_to = async () => {
    const file = await open({
      multiple: false,
      directory: true,
    });

    if (file) {
      setUzloading(true);
      setTimeout(() => {
        new Promise(async (ok, _reject) => {
          try {
            // 使用 await 等待解压完成，避免异步操作导致状态更新混乱
            ok(await archiveExtract(v, file, password));
          } catch (err) {
            console.log("err :", err);
          } finally {
            setUzloading(false);
          }
        });
      }, 10);
    }
  };

  useEffect(() => {
    win_run_args();
    macos_run_args();
    listenFileDrop();
  }, []);
  return (
    <div className="p-4">
      <PasswordDialog
        isOpen={passwordRequire}
        onClose={() => setPasswordRequire(false)}
        onConfirm={handlePasswordConfirm}
      />

      <div className="flex align-center items-end gap-2">
        <Button
          isDisabled={!zipList.length}
          color="primary"
          onPress={zip_to}
          variant="bordered"
          isLoading={uzLoading}
        >
          解压到
        </Button>
      </div>
      <div>
        {!!zipList.length ? (
          <FileTree width={ww} height={hh} data={zipList} />
        ) : (
          <div
            style={{ width: ww, height: hh - 40 }}
            className="bg-blue-200 flex items-center justify-center text-center"
          >
            <div>
              <span className="p-5  z-10">
                拖拽文件放入
                <span className="p-5 opacity-1">
                  支持zip、tar(gz、xz、bz2)、rar、7z
                </span>
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default App;

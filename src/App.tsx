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
// import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { attachConsole } from "@tauri-apps/plugin-log";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@heroui/react";

function App() {
  const [v, setV] = useState("");
  const [ww, setWW] = useState(1170);
  const [hh, setHH] = useState(850);
  const [uzLoading, setUzloading] = useState(false);
  const [passwordRequire, setPasswordRequire] = useState(false);
  const [extractPasswordRequire, setExtractPasswordRequire] = useState(false);
  const [password, setPassword] = useState("");
  const [zipList, setZipList] = useState<TreeNode<ArchiveEntry>[]>([]);
  const [extractPath, setExtractPath] = useState("");
  const [zipFiles, setZipFiles] = useState<string[]>([]);

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
    attachConsole().then(() => {
      console.log("Console attached");
    });

    // 通知前端已经准备好
    getCurrentWebview()
      .emit("ready", getCurrentWebview().label)
      .catch((err) => {
        console.log("ready::err :", err);
      });

    listen("in-extract", (event) => {
      console.log("in-extract", event.payload);
      if (event.payload) {
        setZipFiles(event.payload as string[]);
      }
    });

    // getCurrentWindow()
    //   .innerSize()
    //   .then(async (res) => {
    //     const scaleFactor = await getCurrentWindow().scaleFactor();
    //     console.log("res :", res, res.width / scaleFactor);
    //     const width = res.width / scaleFactor - 40;
    //     const height = res.height / scaleFactor - 40;
    //     setWW(width || 780);
    //     setHH(height || 600);
    //   });
    // getCurrentWindow().onResized(async (res) => {
    //   const scaleFactor = await getCurrentWindow().scaleFactor();
    //   const width = res.payload.width / scaleFactor - 40;
    //   const height = res.payload.height / scaleFactor - 40;
    //   setWW(width || 780);
    //   setHH(height || 600);
    // });
  }, []);
  const cli_run_args = async () => {
    try {
      const paths = await run_args();
      console.log("path==》 :", paths);
      if (paths) {
        let path = paths[0];
        if (!path) {
          return;
        }
        if (path && path.startsWith("file://")) {
          path = path.replace("file://", "");
        }
        let v_path = decodeURIComponent(path);
        setValue(v_path);
        loadList(v_path, password);
      }
    } catch (err) {
      console.log("win error :", err);
    }
  };

  const listenFileDrop = async () => {
    return getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "drop") {
        const paths = event.payload.paths;
        if (paths.length == 1) {
          if (paths[0]) {
            setValue(paths[0]);
            loadList(paths[0], password);
          }
        } else {
          console.log("event :", paths);
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

  const handleExtractPasswordConfirm = (password: string) => {
    setPassword(password);
    setExtractPasswordRequire(false);
    extract(extractPath, password);
  };

  const extract = (extractPath: string, password: string) => {
    setUzloading(true);
    setTimeout(() => {
      new Promise(async (ok, _reject) => {
        archiveExtract(v, extractPath, password)
          .then(ok)
          .catch((err) => {
            if (err.MsgError == "密码错误") {
              setExtractPasswordRequire(true);
            } else {
              alert(JSON.stringify(err));
              console.log("err :", err);
            }
          })
          .finally(() => {
            setUzloading(false);
          });
        alert("解压完成");
      });
    }, 10);
  };
  const zip_to = async () => {
    const file = await open({
      multiple: false,
      directory: true,
    });

    if (file) {
      setExtractPath(file);
      extract(file, password);
    }
  };

  useEffect(() => {
    cli_run_args();
    listenFileDrop();
  }, []);
  return (
    <div className="p-4">
      <PasswordDialog
        isOpen={passwordRequire}
        onClose={() => setPasswordRequire(false)}
        onConfirm={handlePasswordConfirm}
      />

      <PasswordDialog
        isOpen={extractPasswordRequire}
        onClose={() => setExtractPasswordRequire(false)}
        onConfirm={handleExtractPasswordConfirm}
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
      <div className="mt-2">
        {!!zipList.length ? (
          <FileTree width={ww} height={hh} data={zipList} />
        ) : (
          <div
            style={{ width: ww, height: hh }}
            className="bg-blue-200 flex items-center self-center justify-center text-center"
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

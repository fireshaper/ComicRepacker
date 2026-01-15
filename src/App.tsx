import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, Play, Loader2, RefreshCw, XCircle } from 'lucide-react';
import './styles.css';

interface ArchiveInfo {
  file_type: string;
  is_solid: boolean;
  is_encrypted: boolean;
  image_count: number;
  unsupported_reason?: string;
}

interface ScanResult {
  path: string;
  info?: ArchiveInfo;
  error?: string;
  status: string; // "Unsupported", "Supported", "Error", "Pending", "Converting", "Converted"
}

function App() {
  const [directory, setDirectory] = useState<string>('');
  const [scanning, setScanning] = useState(false);
  const [results, setResults] = useState<ScanResult[]>([]);
  const [scannedCount, setScannedCount] = useState(0);
  const [filterUnsupported, setFilterUnsupported] = useState(false);

  // Conversion state
  const [convertingMap, setConvertingMap] = useState<Record<string, boolean>>({});

  useEffect(() => {
    // Listeners
    const unlistenProgress = listen<number>('scan-progress', (event) => {
      setScannedCount(event.payload);
    });

    const unlistenResult = listen<ScanResult>('scan-result', (event) => {
      setResults((prev) => [...prev, event.payload]);
    });

    const unlistenComplete = listen('scan-complete', () => {
      setScanning(false);
    });

    const unlistenCancelled = listen('scan-cancelled', () => {
      setScanning(false);
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenResult.then((f) => f());
      unlistenComplete.then((f) => f());
      unlistenCancelled.then((f) => f());
    };
  }, []);

  const handleSelectDir = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected) {
      setDirectory(selected as string);
      setResults([]);
      setScannedCount(0);
    }
  };

  const handleScan = async () => {
    if (!directory) return;
    setScanning(true);
    setResults([]);
    setScannedCount(0);
    try {
      await invoke('scan_directory', { path: directory });
    } catch (e) {
      console.error(e);
      setScanning(false);
    }
  };

  const handleCancel = async () => {
    await invoke('cancel_scan');
  };

  const handleConvert = async (path: string) => {
    setConvertingMap(prev => ({ ...prev, [path]: true }));
    try {
      await invoke('convert_book', { path });
      // Update status locally
      setResults(prev => prev.map(r => r.path === path ? { ...r, status: "Converted" } : r));
    } catch (e) {
      console.error(e);
      setResults(prev => prev.map(r => r.path === path ? { ...r, status: "Error", error: String(e) } : r));
    } finally {
      setConvertingMap(prev => {
        const next = { ...prev };
        delete next[path];
        return next;
      });
    }
  };

  const handleConvertAll = async () => {
    const unsupported = results.filter(r => r.status === "Unsupported");
    for (const item of unsupported) {
      await handleConvert(item.path);
    }
  };

  const filteredResults = filterUnsupported
    ? results.filter(r => r.status === "Unsupported" || r.status === "Converted" || r.status === "Error")
    : results;

  return (
    <div className="container">
      <div className="header">
        <h1>ComicRepacker</h1>
        <div style={{ display: 'flex', gap: '8px' }}>
          {scanning && (
            <button className="btn" style={{ backgroundColor: 'var(--error)' }} onClick={handleCancel}>
              <XCircle size={16} /> Cancel Scan
            </button>
          )}
        </div>
      </div>

      <div className="scanner-card">
        <button className="btn btn-secondary" onClick={handleSelectDir}>
          <FolderOpen size={18} />
          Select Folder
        </button>
        <div className="path-display" title={directory}>
          {directory || "No directory selected"}
        </div>
        <button
          className="btn"
          onClick={handleScan}
          disabled={!directory || scanning}
        >
          {scanning ? <Loader2 className="spin" size={18} /> : <Play size={18} />}
          {scanning ? 'Scanning...' : 'Scan'}
        </button>
      </div>

      {results.length > 0 && (
        <div className="actions-bar" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
              <input type="checkbox" checked={filterUnsupported} onChange={e => setFilterUnsupported(e.target.checked)} />
              Show only unsupported
            </label>
          </div>
          <div>
            <button className="btn" onClick={handleConvertAll} disabled={scanning}>
              <RefreshCw size={16} /> Convert All Unsupported
            </button>
          </div>
        </div>
      )}

      <div className="results-container">
        <div className="table-header">
          <div>File Path</div>
          <div>Type</div>
          <div>Solid</div>
          <div>Images</div>
          <div>Status</div>
          <div>Action</div>
        </div>
        <div className="table-body">
          {filteredResults.map((res) => {
            const isUnsupported = res.status === "Unsupported";
            const isConverting = convertingMap[res.path];
            const fileName = res.path.split(/[/\\]/).pop();

            return (
              <div className="row" key={res.path}>
                <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={res.path}>
                  {fileName}
                  <div style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>{res.path}</div>
                </div>
                <div>{res.info?.file_type || '-'}</div>
                <div>
                  {res.info?.is_solid && <span className="badge badge-solid">SOLID</span>}
                </div>
                <div>{res.info?.image_count || 0}</div>
                <div>
                  {res.status === "Unsupported" && <span className="badge badge-rar5">Unsupported</span>}
                  {res.status === "Supported" && <span className="badge badge-ok">OK</span>}
                  {res.status === "Converted" && <span className="status-converted">Converted</span>}
                  {res.status === "Error" && (
                    <div style={{ color: 'var(--error)', fontSize: '12px', lineHeight: '1.2' }}>
                      <span className="status-error" style={{ marginBottom: '4px', display: 'inline-block' }}>Error</span>
                      <div>{res.error}</div>
                    </div>
                  )}
                </div>
                <div>
                  {isConverting ? (
                    <div style={{ width: '80px', display: 'flex', flexDirection: 'column', gap: '4px', alignItems: 'center' }}>
                      <div className="progress-bar-container">
                        <div className="progress-bar-value"></div>
                      </div>
                      <span style={{ fontSize: '10px', color: 'var(--text-secondary)' }}>Converting...</span>
                    </div>
                  ) : (
                    isUnsupported && (
                      <button
                        className="btn btn-secondary"
                        style={{ padding: '4px 8px', fontSize: '12px' }}
                        onClick={() => handleConvert(res.path)}
                      >
                        Convert
                      </button>
                    )
                  )}
                </div>
              </div>
            );
          })}
        </div>
        <div style={{ padding: '8px 24px', borderTop: '1px solid var(--border)', fontSize: '12px', color: 'var(--text-secondary)' }}>
          Found: {scannedCount} | Unsupported: {results.filter(r => r.status === "Unsupported").length}
        </div>
      </div>
    </div >
  );
}

export default App;

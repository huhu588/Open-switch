import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Download, Upload, Loader2, Shield } from 'lucide-react';
import { sub2apiClient } from '../../services/sub2apiClient';
import { useToast } from '../../hooks/useToast';
import { ToastContainer } from '../../components/Toast';

export default function Sub2apiBackup() {
  const { t } = useTranslation();
  const toast = useToast();
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);

  const handleExport = async () => {
    setExporting(true);
    try {
      const data = await sub2apiClient.post<Record<string, unknown>>('/admin/backups');
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `sub2api-backup-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      toast.success(t('sub2api.backup.exportSuccess', '备份已创建'));
    } catch (err) {
      toast.error(String(err));
    } finally {
      setExporting(false);
    }
  };

  const handleImport = async () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      setImporting(true);
      try {
        const text = await file.text();
        const json = JSON.parse(text);
        await sub2apiClient.post('/admin/backups', json);
        toast.success(t('sub2api.backup.importSuccess', '备份已恢复'));
      } catch (err) {
        toast.error(String(err));
      } finally {
        setImporting(false);
      }
    };
    input.click();
  };

  return (
    <div>
      <ToastContainer toasts={toast.toasts} />
      <h2 className="s2a-page-title">{t('sub2api.backup.title', '备份与恢复')}</h2>

      <div className="s2a-stats-grid" style={{ maxWidth: 600 }}>
        <div className="s2a-stat-card" style={{ cursor: 'pointer' }} onClick={handleExport}>
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.backup.export', '导出备份')}</span>
            {exporting ? <Loader2 size={16} className="gw-spin s2a-stat-card-icon" /> : <Download size={16} className="s2a-stat-card-icon" />}
          </div>
          <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
            {t('sub2api.backup.exportDesc', '将所有数据导出为 JSON 文件')}
          </div>
        </div>
        <div className="s2a-stat-card" style={{ cursor: 'pointer' }} onClick={handleImport}>
          <div className="s2a-stat-card-header">
            <span className="s2a-stat-card-label">{t('sub2api.backup.import', '恢复备份')}</span>
            {importing ? <Loader2 size={16} className="gw-spin s2a-stat-card-icon" /> : <Upload size={16} className="s2a-stat-card-icon" />}
          </div>
          <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
            {t('sub2api.backup.importDesc', '从 JSON 文件恢复数据')}
          </div>
        </div>
      </div>

      <div className="s2a-section" style={{ marginTop: 20 }}>
        <div className="s2a-section-title" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Shield size={16} />
          {t('sub2api.backup.tips', '注意事项')}
        </div>
        <ul style={{ fontSize: '0.72rem', color: 'var(--text-muted)', paddingLeft: 20, lineHeight: 2 }}>
          <li>{t('sub2api.backup.tip1', '备份包含所有账号、用户、API Key、分组等数据')}</li>
          <li>{t('sub2api.backup.tip2', '恢复操作会覆盖现有数据，请谨慎操作')}</li>
          <li>{t('sub2api.backup.tip3', '建议定期备份重要数据')}</li>
        </ul>
      </div>
    </div>
  );
}

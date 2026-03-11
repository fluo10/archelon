import * as path from 'path';
import * as vscode from 'vscode';
import { EntryRecord, treeEntries } from './cli';
import { findJournalRoot } from './journal';

export class EntryItem extends vscode.TreeItem {
    constructor(
        public readonly record: EntryRecord,
        public readonly children: EntryRecord[],
    ) {
        super(
            record.title || '(untitled)',
            children.length > 0
                ? vscode.TreeItemCollapsibleState.Collapsed
                : vscode.TreeItemCollapsibleState.None,
        );

        this.command = {
            command: 'vscode.open',
            title: 'Open Entry',
            arguments: [vscode.Uri.file(record.path)],
        };

        if (record.task) {
            this.description = `[${record.task.status}]`;
        } else if (record.event) {
            this.description = record.event.start === record.event.end
                ? record.event.start.slice(0, 10)
                : `${record.event.start.slice(0, 10)} – ${record.event.end.slice(0, 10)}`;
        }

        const tagPart = record.tags.length > 0 ? `\nTags: #${record.tags.join(' #')}` : '';
        this.tooltip = `${record.id}${tagPart}`;
        this.contextValue = 'entry';
    }
}

export class EntryTreeProvider implements vscode.TreeDataProvider<EntryItem> {
    private _onDidChangeTreeData = new vscode.EventEmitter<EntryItem | undefined | void>();
    readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

    private _filter = '';
    private _rootRecords: EntryRecord[] = [];

    get filter(): string { return this._filter; }

    refresh(): void {
        this._rootRecords = [];
        this._onDidChangeTreeData.fire();
    }

    setFilter(text: string): void {
        this._filter = text;
        this._rootRecords = [];
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: EntryItem): vscode.TreeItem {
        return element;
    }

    async getChildren(element?: EntryItem): Promise<EntryItem[]> {
        if (element) {
            return this._toItems(element.children);
        }

        const cwd = this._getCwd();
        if (!cwd) { return []; }

        try {
            this._rootRecords = await treeEntries(cwd);
        } catch {
            return [];
        }

        let roots = this._rootRecords;
        if (this._filter) {
            roots = this._filterRecords(roots, this._filter.toLowerCase());
        }

        return this._toItems(roots);
    }

    private _toItems(records: EntryRecord[]): EntryItem[] {
        return records.map(r => new EntryItem(r, r.children ?? []));
    }

    /** Recursively keep records whose title/id/tags match, preserving matched subtrees. */
    private _filterRecords(records: EntryRecord[], f: string): EntryRecord[] {
        const result: EntryRecord[] = [];
        for (const r of records) {
            const selfMatch =
                r.title.toLowerCase().includes(f) ||
                r.id.toLowerCase().includes(f) ||
                r.tags.some(t => t.toLowerCase().includes(f));
            const filteredChildren = this._filterRecords(r.children ?? [], f);
            if (selfMatch || filteredChildren.length > 0) {
                result.push({ ...r, children: filteredChildren });
            }
        }
        return result;
    }

    private _getCwd(): string | null {
        const activeFile = vscode.window.activeTextEditor?.document.uri.fsPath;
        if (activeFile && findJournalRoot(activeFile)) {
            return path.dirname(activeFile);
        }
        return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? null;
    }
}

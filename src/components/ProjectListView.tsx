import React from 'react';
import { Project } from '../types';
import { Plus, BookOpen, Trash2, Copy, Check, Clock, ArrowRight } from 'lucide-react';

interface ProjectListViewProps {
  projects: Project[];
  activeProjectId: string | null;
  onSelectProject: (projectId: string) => void;
  onOpenDetail: (projectId: string) => void;
  onCreateNewProject: () => void;
  onDuplicateProject: (projectId: string) => void;
  onDeleteProject: (projectId: string) => void;
}

export const ProjectListView: React.FC<ProjectListViewProps> = ({
  projects,
  activeProjectId,
  onSelectProject,
  onOpenDetail,
  onCreateNewProject,
  onDuplicateProject,
  onDeleteProject,
}) => {
  return (
    <div className="max-w-7xl mx-auto p-6 space-y-8">
      {/* 画面最上部: タイトル ＆ 「+ 新規作品を作成」ボタン */}
      <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 shadow-sm flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div>
          <div className="flex items-center space-x-3 mb-1">
            <BookOpen className="w-6 h-6 text-indigo-400" />
            <h2 className="text-xl font-bold text-slate-100">作品一覧 (小説プロジェクト)</h2>
          </div>
          <p className="text-slate-400 text-xs">
            保存されている作品の一覧です。カードをクリックして作品詳細画面を開きます。
          </p>
        </div>

        <button
          onClick={onCreateNewProject}
          className="flex items-center space-x-2 px-6 py-3 rounded-xl bg-gradient-to-r from-indigo-600 via-purple-600 to-pink-600 hover:from-indigo-500 hover:to-pink-500 text-white font-bold text-sm shadow-lg shadow-indigo-600/30 transition-all transform hover:scale-105"
        >
          <Plus className="w-5 h-5" />
          <span>新規作品を作成</span>
        </button>
      </div>

      {/* 作品カードグリッド */}
      <div className="space-y-3">
        <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider">
          保存された作品 ({projects.length} 件)
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
          {projects.map((proj) => {
            const isSelected = proj.id === activeProjectId;
            const completedChCount =
              proj.novelData?.chapters.filter(
                (c) => c.status === 'completed' || (c.scenes.length > 0 && c.scenes.every((sc) => sc.wordCount > 0))
              ).length || 0;
            const totalWords = proj.novelData?.totalWordCount || 0;
            const targetChCount =
              proj.promptSettings?.targetChapterCount ||
              (proj.novelData?.chapters.length && proj.novelData.chapters.length > 0
                ? proj.novelData.chapters.length
                : 12);
            const isCompleted = completedChCount >= targetChCount && completedChCount > 0;

            const handleCardClick = () => {
              onSelectProject(proj.id);
              onOpenDetail(proj.id);
            };

            return (
              <div
                key={proj.id}
                onClick={handleCardClick}
                className={`group bg-slate-900 border rounded-2xl p-5 space-y-4 cursor-pointer transition-all relative flex flex-col justify-between hover:scale-[1.01] ${
                  isSelected
                    ? isCompleted
                      ? 'border-emerald-500 shadow-xl shadow-emerald-950/60 ring-1 ring-emerald-500'
                      : 'border-indigo-500 shadow-xl shadow-indigo-950/60 ring-1 ring-indigo-500'
                    : isCompleted
                    ? 'border-emerald-800/80 hover:border-emerald-700 hover:bg-slate-900/90'
                    : 'border-slate-800 hover:border-slate-700 hover:bg-slate-900/90'
                }`}
              >
                <div className="space-y-2.5">
                  <div className="flex justify-between items-start">
                    <div className="flex items-center space-x-2 flex-1 mr-2 min-w-0">
                      <h4 className="font-bold text-base text-slate-100 line-clamp-1 group-hover:text-indigo-300 transition-colors">
                        {proj.novelData?.title || proj.title || '無題作品'}
                      </h4>
                      {isCompleted && (
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-emerald-950 text-emerald-300 border border-emerald-700/80 shrink-0 flex items-center space-x-0.5 shadow-sm">
                          <Check className="w-3 h-3 text-emerald-400" />
                          <span>全話完成</span>
                        </span>
                      )}
                    </div>

                    <div className="flex items-center space-x-1 shrink-0">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onDuplicateProject(proj.id);
                        }}
                        className="p-1.5 rounded-lg bg-slate-800/80 hover:bg-indigo-950 hover:text-indigo-300 text-slate-400 border border-slate-700/60 hover:border-indigo-600 transition-colors flex items-center space-x-1 text-xs"
                        title="お題のキーワードのみをコピーして新規作品を複製登録します"
                      >
                        <Copy className="w-3.5 h-3.5" />
                        <span className="text-[11px] font-medium hidden sm:inline">複製</span>
                      </button>

                      {projects.length > 1 && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            if (confirm(`作品「${proj.title}」を削除しますか？`)) {
                              onDeleteProject(proj.id);
                            }
                          }}
                          className="p-1.5 rounded-lg bg-slate-800/80 hover:bg-rose-950 hover:text-rose-400 text-slate-400 border border-slate-700/60 hover:border-rose-800 transition-colors"
                          title="作品削除"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </div>
                  </div>

                  <p className="text-xs text-slate-400 line-clamp-2 leading-relaxed font-sans min-h-[32px]">
                    {proj.promptSettings?.storyConcept || 'お題未設定'}
                  </p>

                  <div className="flex flex-wrap gap-1.5 pt-1">
                    {proj.promptSettings?.themes.map((t) => (
                      <span
                        key={t}
                        className="px-2 py-0.5 rounded text-[10px] bg-slate-950 border border-slate-800 text-indigo-300"
                      >
                        #{t}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="border-t border-slate-800/80 pt-3 space-y-2.5">
                  <div className="flex items-center justify-between text-xs font-mono">
                    <span
                      className={
                        isCompleted
                          ? 'text-emerald-400 font-bold flex items-center space-x-1'
                          : 'text-slate-400'
                      }
                    >
                      <span>進捗:</span>
                      <strong className={isCompleted ? 'text-emerald-300 font-bold' : 'text-indigo-300'}>
                        {completedChCount}
                      </strong>{' '}
                      / {targetChCount} 話
                      {isCompleted && (
                        <span className="text-[10px] bg-emerald-950 text-emerald-300 px-1.5 py-0.5 rounded border border-emerald-800 font-sans ml-1">
                          🎉 完成
                        </span>
                      )}
                    </span>
                    <span className="text-slate-400">
                      <strong className="text-emerald-400">{totalWords.toLocaleString()}</strong> 文字
                    </span>
                  </div>

                  <div className="flex items-center justify-between pt-1">
                    <span className="text-[10px] text-slate-500 flex items-center space-x-1">
                      <Clock className="w-3 h-3" />
                      <span>更新日: {proj.lastUpdatedDate}</span>
                    </span>

                    <span className="text-xs font-semibold text-indigo-400 group-hover:text-indigo-300 flex items-center space-x-1 transition-colors">
                      <span>詳細を見る</span>
                      <ArrowRight className="w-3.5 h-3.5 transform group-hover:translate-x-0.5 transition-transform" />
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

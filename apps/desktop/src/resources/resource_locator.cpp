#include "resource_locator.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QProcessEnvironment>

namespace {

QString findRepoRoot(const QDir& fromDir) {
    QDir cursor(fromDir);
    while (true) {
        if (QFileInfo::exists(cursor.filePath(".git"))) {
            return cursor.absolutePath();
        }
        if (!cursor.cdUp()) {
            break;
        }
    }
    return {};
}

} // namespace

QStringList resolveRequired(const QDir& root) {
    QStringList missing;
    if (!QFileInfo::exists(root.filePath("templates")) || !QFileInfo(root.filePath("templates")).isDir()) {
        missing << "templates";
    }
    if (!QFileInfo::exists(root.filePath("samples")) || !QFileInfo(root.filePath("samples")).isDir()) {
        missing << "samples";
    }
    return missing;
}

ResourceRoot locateResourceRoot() {
    ResourceRoot out;

    const QProcessEnvironment env = QProcessEnvironment::systemEnvironment();
    const QString exeDirPath = QCoreApplication::applicationDirPath();
    const QDir exeDir(exeDirPath);

    const QString envRoot = env.value("CRAFTCAD_RESOURCE_ROOT");
    if (!envRoot.isEmpty()) {
        out.searched << envRoot;
        QFileInfo fi(envRoot);
        if (!fi.isAbsolute()) {
            out.reasonCode = "RESOURCE_ROOT_NOT_FOUND";
            out.message = "CRAFTCAD_RESOURCE_ROOT must be absolute";
            return out;
        }
        if (!fi.exists() || !fi.isDir()) {
            out.reasonCode = "RESOURCE_ROOT_NOT_FOUND";
            out.message = "CRAFTCAD_RESOURCE_ROOT does not exist";
            return out;
        }
        out.root = QDir(fi.absoluteFilePath());
        out.missingRequired = resolveRequired(out.root);
        if (!out.missingRequired.isEmpty()) {
            out.reasonCode = "RESOURCE_MISSING";
            out.message = "required resource directories are missing";
            return out;
        }
        out.ok = true;
        return out;
    }

    const QString exeRelative = exeDir.filePath("resources");
    out.searched << exeRelative;
    if (QFileInfo::exists(exeRelative) && QFileInfo(exeRelative).isDir()) {
        out.root = QDir(exeRelative);
        out.missingRequired = resolveRequired(out.root);
        if (!out.missingRequired.isEmpty()) {
            out.reasonCode = "RESOURCE_MISSING";
            out.message = "required resource directories are missing";
            return out;
        }
        out.ok = true;
        return out;
    }

    const QString repoRoot = findRepoRoot(exeDir);
    if (!repoRoot.isEmpty()) {
        const QString devFallback = QDir(repoRoot).filePath("apps/desktop/resources");
        out.searched << devFallback;
        if (QFileInfo::exists(devFallback) && QFileInfo(devFallback).isDir()) {
            out.root = QDir(devFallback);
            out.missingRequired = resolveRequired(out.root);
            if (!out.missingRequired.isEmpty()) {
                out.reasonCode = "RESOURCE_MISSING";
                out.message = "required resource directories are missing";
                return out;
            }
            out.ok = true;
            return out;
        }
    }

    out.reasonCode = "RESOURCE_ROOT_NOT_FOUND";
    out.message = "resource root not found";
    return out;
}

QString describeResourceRoot(const ResourceRoot& root) {
    if (root.ok) {
        return QStringLiteral("ok root=%1").arg(root.root.absolutePath());
    }
    return QStringLiteral("error reason=%1 message=%2 searched=%3")
        .arg(root.reasonCode, root.message, root.searched.join(","));
}

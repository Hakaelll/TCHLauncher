/*
 * Installer
 * Copyright (c) 2016-2018.
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation version 2.1
 * of the License.
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Lesser General Public License for more details.
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
 */
package net.minecraftforge.installer.actions;

import java.io.File;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Predicate;
import net.minecraftforge.installer.DownloadUtils;
import net.minecraftforge.installer.SimpleInstaller;
import net.minecraftforge.installer.json.Artifact;
import net.minecraftforge.installer.json.InstallV1;
import net.minecraftforge.installer.json.Util;
import net.minecraftforge.installer.json.Version;
import net.minecraftforge.installer.json.Version.Download;
import net.minecraftforge.installer.ui.TranslatedMessage;

public class ServerInstall extends Action {
    public static boolean serverStarterJar;

    private final List<Artifact> grabbed = new ArrayList<>();

    public ServerInstall(InstallV1 profile, ProgressCallback monitor) {
        super(profile, monitor, false);
    }

    @Override
    public boolean run(File target, Predicate<String> optionals, File installer) throws ActionCanceledException {
        if (target.exists() && !target.isDirectory()) {
            error("There is a file at this location, the server cannot be installed here!");
            return false;
        }

        File librariesDir = new File(target, "libraries");
        if (!target.exists())
            target.mkdirs();
        librariesDir.mkdir();
        if (profile.getMirror() != null && profile.getMirror().isAdvertised())
            monitor.stage(getSponsorMessage());
        checkCancel();

        //Download MC Server jar
        monitor.stage("Considering Minecraft server jar", true);
        Map<String, String> tokens = new HashMap<>();
        tokens.put("ROOT", target.getAbsolutePath());
        tokens.put("MINECRAFT_VERSION", profile.getMinecraft());
        tokens.put("LIBRARY_DIR", librariesDir.getAbsolutePath());

        String path = Util.replaceTokens(tokens, profile.getServerJarPath());
        File serverTarget = new File(path);
        if (!serverTarget.exists()) {
            File parent = serverTarget.getParentFile();
            if (!parent.exists()) {
                parent.mkdirs();
            }

            File versionJson = new File(target, profile.getMinecraft() + ".json");
            Version vanilla = Util.getVanillaVersion(monitor, profile.getMinecraft(), versionJson);
            if (vanilla == null) {
                error("Failed to download version manifest, can not find server jar URL.");
                return false;
            }
            Download server = vanilla.getDownload("server");
            if (server == null) {
                error("Failed to download minecraft server, info missing from manifest: " + versionJson);
                return false;
            }

            versionJson.delete();

            if (!monitor.downloader(server.getUrl())
                    .sha(server.getSha1())
                    .localPath("minecraft/" + profile.getMinecraft() + "/server.jar")
                    .download(serverTarget)) {
                serverTarget.delete();
                error("Downloading minecraft server failed, invalid checksum.\n" +
                        "Try again, or manually place server jar to skip download.");
                return false;
            }
        }
        checkCancel();

        // Download Libraries
        List<File> libDirs = new ArrayList<>();
        File mcLibDir = new File(SimpleInstaller.getMCDir(), "libraries");
        if (mcLibDir.exists()) {
            libDirs.add(mcLibDir);
        }
        if (!downloadLibraries(librariesDir, optionals, libDirs))
            return false;

        checkCancel();
        if (!processors.process(librariesDir, serverTarget, target, installer))
            return false;

        if (serverStarterJar) {
            monitor.downloader(DownloadUtils.SERVER_STARTER_JAR)
                    .localPath("serverstarter.jar")
                    .download(new File(target, "server.jar"));
        }

        return true;
    }

    @Override
    public TargetValidator getTargetValidator() {
        return TargetValidator.shouldExist(false)
                .and(TargetValidator.isDirectory())
                .and(TargetValidator.shouldBeEmpty());
    }

    @Override
    public TranslatedMessage getSuccessMessage() {
        if (grabbed.isEmpty()) {
            return new TranslatedMessage("installer.action.install.server.finished.withoutlibs", profile.getVersion());
        }
        return new TranslatedMessage("installer.action.install.server.finished.withlibs", grabbed.size(), profile.getVersion());
    }
}

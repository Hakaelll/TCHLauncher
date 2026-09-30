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
package net.minecraftforge.installer;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.net.URISyntaxException;
import java.net.URL;
import java.net.URLConnection;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.util.Objects;
import java.util.logging.Level;
import java.util.logging.Logger;
import net.minecraftforge.installer.actions.ProgressCallback;
import org.jetbrains.annotations.CheckReturnValue;
import org.jetbrains.annotations.Nullable;

@CheckReturnValue
public class Downloader {
    private static final Logger LOGGER = Logger.getLogger("Downloading");
    public static final LocalSource LOCAL = LocalSource.detect();

    private LocalSource localSource;
    private final ProgressCallback monitor;
    private final String url;
    private String sha1, localPath;

    public Downloader(LocalSource localSource, ProgressCallback monitor, String url) {
        this.localSource = localSource;
        this.monitor = monitor;
        this.url = url;
    }

    public Downloader sha(@Nullable String sha) {
        this.sha1 = sha;
        return this;
    }

    public Downloader localPath(@Nullable String localPath) {
        this.localPath = localPath;
        return this;
    }

    public Downloader additionalDirectory(File... dirs) {
        for (File dir : dirs) {
            this.localSource = this.localSource.fallbackWith(LocalSource.fromDir(dir.toPath()));
        }
        return this;
    }

    public boolean download(File target) {
        if (target.exists() && this.sha1 != null) {
            if (Objects.equals(this.sha1, DownloadUtils.getSha1(target))) {
                monitor.message("File " + target + " exists. Checksum valid.");
                return true;
            } else {
                monitor.message("File " + target + " exists. Invalid checksum, deleting file.");
                target.delete();
            }
        }

        Path nio = target.toPath();
        try {
            if (nio.getParent() != null) Files.createDirectories(nio.getParent());
        } catch (IOException exception) {
            LOGGER.log(Level.WARNING, exception, () -> "Failed to create parent of " + target);
            return false;
        }

        if (localPath != null) {
            try {
                LocalFile alternative = this.localSource.getArtifact(localPath);
                if (alternative != null) {
                    Files.copy(alternative.stream, nio, StandardCopyOption.REPLACE_EXISTING);
                    if (this.sha1 != null) {
                        String actualSha = DownloadUtils.getSha1(target);
                        if (!Objects.equals(actualSha, this.sha1)) {
                            monitor.message("Invalid checksum. Downloaded locally from " + alternative.path);
                            monitor.message("\tExpected: " + this.sha1);
                            monitor.message("\tActual:   " + actualSha);
                        } else {
                            monitor.message("Downloaded file locally from " + alternative.path + ", valid checksum.");
                            return true;
                        }
                    } else {
                        monitor.message("Downloaded file locally from " + alternative.path + ", no checksum provided, assuming valid.");
                        return true;
                    }
                }
            } catch (IOException exception) {
                LOGGER.log(Level.WARNING, exception, () -> "Failed to download from local download source");
            }
        }

        if (DownloadUtils.OFFLINE_MODE) {
            monitor.message("\tFound no cached library at " + target + ", expecting download from " + url + ", but running in offline mode.");
            return false;
        }

        monitor.message("Downloading library from " + url);
        try {
            URLConnection connection = DownloadUtils.getConnection(url);
            if (connection != null) {
                Files.copy(monitor.wrapStepDownload(connection), target.toPath(), StandardCopyOption.REPLACE_EXISTING);

                if (this.sha1 != null) {
                    String sha1 = DownloadUtils.getSha1(target);
                    if (Objects.equals(sha1, this.sha1)) {
                        monitor.message("\tDownload completed: Checksum validated.");
                        return true;
                    }
                    monitor.message("\tDownload failed: Checksum invalid, deleting file:");
                    monitor.message("\t\tExpected: " + this.sha1);
                    monitor.message("\t\tActual:   " + sha1);
                    if (!target.delete()) {
                        monitor.stage("\tFailed to delete file, aborting.");
                        return false;
                    }
                } else {
                    monitor.message("\tDownload completed: No checksum, Assuming valid.");
                    return true;
                }
            }
        } catch (IOException e) {
            LOGGER.log(Level.WARNING, e, () -> "Failed to download from " + url);
        }

        return false;
    }

    public InputStream openStream() throws IOException {
        if (localPath != null) {
            LocalFile alternative = this.localSource.getArtifact(localPath);
            if (alternative != null) {
                return alternative.stream;
            }
        }

        if (DownloadUtils.OFFLINE_MODE) {
            monitor.message("\tLibrary not cached, expecting download from " + url + ", but running in offline mode.");
            throw new RuntimeException("Running in offline mode, cannot download from " + url + ", cached version not found");
        }
        return monitor.wrapStepDownload(DownloadUtils.getConnection(url));
    }

    public static class LocalFile {
        public final InputStream stream;
        public final String path;

        public LocalFile(InputStream stream, String path) {
            this.stream = stream;
            this.path = path;
        }
    }

    public interface LocalSource {
        @Nullable
        LocalFile getArtifact(String path) throws IOException;

        default LocalSource fallbackWith(@Nullable LocalSource other) {
            if (other == null) {
                return this;
            }

            return p -> {
                final LocalFile art = getArtifact(p);
                return art != null ? art : other.getArtifact(p);
            };
        }

        static LocalSource walkFromClassesOut(Path out) {
            // The local path would be LegacyInstaller/build/classes/java/main, so walk upwards 4 times
            for (int i = 0; i < 3; i++) {
                out = out.getParent();
            }

            // The maven src dir is in src/main/resources/maven
            final Path base = out.resolve("src/main/resources/maven");
            return fromDir(base);
        }

        @Nullable
        static LocalSource walkFromLibs(Path source) {
            source = source.getParent();
            if (source == null || source.getFileName() == null || !source.getFileName().toString().equals("libs")) return null;
            source = source.getParent();
            if (source == null || source.getFileName() == null || !source.getFileName().toString().equals("build")) return null;
            source = source.getParent();
            if (source == null) return null;

            final Path base = source.resolve("src/main/resources/maven");
            return Files.isDirectory(base) ? fromDir(base) : null;
        }

        static LocalSource fromDir(Path base) {
            return p -> {
                final Path children = base.resolve(p);
                try {
                    return new LocalFile(Files.newInputStream(children), children.toFile().getAbsolutePath());
                } catch (NoSuchFileException ex) {
                    return null;
                }
            };
        }

        static LocalSource fromResource() {
            return p -> {
                InputStream is = DownloadUtils.class.getResourceAsStream("/maven/" + p);
                return is == null ? null : new LocalFile(is, "jar:/maven/" + p);
            };
        }

        static LocalSource detect() {
            try {
                final URL url = DownloadUtils.class.getProtectionDomain().getCodeSource().getLocation();
                if (url.getProtocol().equals("file") && Files.isDirectory(Paths.get(url.toURI()))) { // If we're running local IDE, use the resources dir
                    return walkFromClassesOut(Paths.get(url.toURI()));
                }

                return fromResource().fallbackWith(walkFromLibs(Paths.get(url.toURI())));
            } catch (URISyntaxException e) {
                throw new RuntimeException(e);
            }
        }
    }
}

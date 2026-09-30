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
import java.net.HttpURLConnection;
import java.net.InetAddress;
import java.net.MalformedURLException;
import java.net.URL;
import java.net.URLConnection;
import java.net.UnknownHostException;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import javax.net.ssl.SSLHandshakeException;
import net.minecraftforge.installer.actions.ProgressCallback;
import net.minecraftforge.installer.json.Artifact;
import net.minecraftforge.installer.json.InstallV1;
import net.minecraftforge.installer.json.Manifest;
import net.minecraftforge.installer.json.Mirror;
import net.minecraftforge.installer.json.Util;
import net.minecraftforge.installer.json.Version.Library;
import net.minecraftforge.installer.json.Version.LibraryDownload;
import org.jetbrains.annotations.Nullable;

public class DownloadUtils {
    public static final String MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
    public static final String SERVER_STARTER_JAR = "https://github.com/NeoForged/serverstarterjar/releases/latest/download/server.jar";

    @Nullable
    public static String userAgent;

    public static boolean OFFLINE_MODE = false;

    public static boolean downloadLibrary(ProgressCallback monitor, Library library, File root, Predicate<String> optional, List<Artifact> grabbed, List<File> additionalLibraryDirs) {
        Artifact artifact = library.getName();
        File target = artifact.getLocalPath(root);
        LibraryDownload download = library.getDownloads() == null ? null : library.getDownloads().getArtifact();
        if (download == null) {
            download = new LibraryDownload();
            download.setPath(artifact.getPath());
        }

        if (!optional.test(library.getName().getDescriptor())) {
            monitor.message(String.format("Considering library %s: Not Downloading {Disabled}", artifact.getDescriptor()));
            return true;
        }

        monitor.message(String.format("Considering library %s", artifact.getDescriptor()));

        String url = download.getUrl();
        if (url == null || url.isEmpty()) {
            monitor.message("  Invalid library, missing url");
            return false;
        }

        if (monitor.downloader(url)
                .additionalDirectory(additionalLibraryDirs.toArray(new File[0]))
                .sha(download.getSha1())
                .localPath(download.getPath())
                .download(target)) {
            grabbed.add(artifact);
            return true;
        }
        return false;
    }

    public static String getSha1(File target) {
        try {
            return HashFunction.SHA1.hash(Files.readAllBytes(target.toPath())).toString();
        } catch (IOException e) {
            e.printStackTrace();
            return null;
        }
    }

    private static boolean checksumValid(File target, String checksum) {
        if (checksum == null || checksum.isEmpty())
            return true;
        String sha1 = getSha1(target);
        return sha1 != null && sha1.equals(checksum);
    }

    public static URLConnection getConnection(String address) {
        if (OFFLINE_MODE) {
            System.out.println("Offline Mode: Not downloading: " + address);
            return null;
        }

        URL url;
        try {
            url = new URL(address);
        } catch (MalformedURLException e) {
            e.printStackTrace();
            return null;
        }

        try {
            int MAX = 3;
            URLConnection connection = null;
            for (int x = 0; x < MAX; x++) { //Maximum of 3 redirects.
                connection = url.openConnection();
                connection.setConnectTimeout(5000);
                connection.setReadTimeout(5000);
                if (connection instanceof HttpURLConnection) {
                    HttpURLConnection hcon = (HttpURLConnection) connection;
                    hcon.setRequestProperty("User-Agent", getUserAgent());
                    hcon.setInstanceFollowRedirects(false);
                    int res = hcon.getResponseCode();
                    if (res == HttpURLConnection.HTTP_MOVED_PERM || res == HttpURLConnection.HTTP_MOVED_TEMP) {
                        String location = hcon.getHeaderField("Location");
                        hcon.disconnect(); //Kill old connection.
                        if (x == MAX - 1) {
                            System.out.println("Invalid number of redirects: " + location);
                            return null;
                        } else {
                            System.out.println("Following redirect: " + location);
                            url = new URL(url, location); // Nested in case of relative urls.
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            return connection;
        } catch (SSLHandshakeException e) {
            System.out.println("Failed to establish connection to " + address);
            String host = url.getHost();
            System.out.println(" Host: " + host + " [" + getIps(host).stream().collect(Collectors.joining(", ")) + "]");
            e.printStackTrace();
            return null;
        } catch (IOException e) {
            e.printStackTrace();
            return null;
        }
    }

    private static String getUserAgent() {
        if (userAgent == null) {
            userAgent = "NeoForged LegacyInstaller";

            String installerVersion = SimpleInstaller.class.getPackage().getImplementationVersion();
            if (installerVersion != null) {
                userAgent += " (" + installerVersion + ")";
            }

            try {
                InstallV1 profile = Util.loadInstallProfile();
                userAgent += " / " + profile.getVersion();
            } catch (Exception ignored) {

            }
        }
        return userAgent;
    }

    public static List<String> getIps(String host) {
        try {
            InetAddress[] addresses = InetAddress.getAllByName(host);
            return Arrays.stream(addresses).map(InetAddress::getHostAddress).collect(Collectors.toList());
        } catch (UnknownHostException e1) {
            e1.printStackTrace();
        }
        return Collections.emptyList();
    }

    public static Mirror[] downloadMirrors(String url) {
        try {
            URLConnection connection = getConnection(url);
            if (connection != null) {
                try (InputStream stream = connection.getInputStream()) {
                    return Util.loadMirrorList(stream);
                }
            }
        } catch (Throwable e) {
            e.printStackTrace();
        }
        return null;
    }

    public static Manifest downloadManifest(ProgressCallback callback) {
        try (InputStream stream = callback.downloader(MANIFEST_URL)
                .localPath("version_manifest.json")
                .openStream()) {
            return Util.loadManifest(stream);
        } catch (IOException e) {
            e.printStackTrace();
        }
        return null;
    }

    public static boolean downloadFile(File target, String url) {
        try {
            URLConnection connection = getConnection(url);
            if (connection != null) {
                Files.copy(connection.getInputStream(), target.toPath(), StandardCopyOption.REPLACE_EXISTING);
                return true;
            }
        } catch (IOException e) {
            e.printStackTrace();
        }
        return false;
    }

    public static boolean extractFile(Artifact art, File target, String checksum) {
        final InputStream input = DownloadUtils.class.getResourceAsStream("/maven/" + art.getPath());
        if (input == null) {
            System.out.println("File not found in installer archive: /maven/" + art.getPath());
            return false;
        }

        if (!target.getParentFile().exists())
            target.getParentFile().mkdirs();

        try {
            Files.copy(input, target.toPath(), StandardCopyOption.REPLACE_EXISTING);
            return checksumValid(target, checksum);
        } catch (Exception e) {
            e.printStackTrace();
            return false;
        }
    }

    public static boolean extractFile(String name, File target) {
        final String path = name.charAt(0) == '/' ? name : '/' + name;
        final InputStream input = DownloadUtils.class.getResourceAsStream(path);
        if (input == null) {
            System.out.println("File not found in installer archive: " + path);
            return false;
        }

        if (!target.getParentFile().exists())
            target.getParentFile().mkdirs();

        try {
            Files.copy(input, target.toPath(), StandardCopyOption.REPLACE_EXISTING);
            return true; //checksumValid(target, checksum); //TODO: zip checksums?
        } catch (Exception e) {
            e.printStackTrace();
            return false;
        }
    }
}

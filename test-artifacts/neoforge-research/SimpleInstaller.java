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

import java.awt.GraphicsEnvironment;
import java.io.BufferedOutputStream;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.io.PrintStream;
import java.net.URISyntaxException;
import java.net.URL;
import java.nio.file.Files;
import java.text.SimpleDateFormat;
import java.util.Arrays;
import java.util.Date;
import java.util.Locale;
import java.util.jar.JarFile;
import java.util.regex.Pattern;
import java.util.stream.Collectors;
import javax.swing.JOptionPane;
import javax.swing.UIManager;
import joptsimple.OptionParser;
import joptsimple.OptionSet;
import joptsimple.OptionSpec;
import net.minecraftforge.installer.actions.Actions;
import net.minecraftforge.installer.actions.FatInstallerAction;
import net.minecraftforge.installer.actions.ProgressCallback;
import net.minecraftforge.installer.actions.ServerInstall;
import net.minecraftforge.installer.json.InstallV1;
import net.minecraftforge.installer.json.Util;
import net.minecraftforge.installer.ui.InstallerPanel;
import net.neoforged.cliutils.progress.ProgressInterceptor;
import net.neoforged.cliutils.progress.ProgressManager;
import net.neoforged.cliutils.progress.ProgressReporter;

public class SimpleInstaller {
    public static boolean headless = false;
    public static boolean debug = false;
    public static URL mirror = null;
    public static boolean skipHashCheck = false;

    public static void main(String[] args) throws IOException, URISyntaxException {
        ProgressCallback monitor;
        try {
            monitor = ProgressCallback.withOutputs(System.out, getLog());
        } catch (FileNotFoundException e) {
            e.printStackTrace();
            monitor = ProgressCallback.withOutputs(System.out);
        }
        hookStdOut(monitor);

        if (System.getProperty("java.net.preferIPv4Stack") == null) //This is a dirty hack, but screw it, i'm hoping this as default will fix more things then it breaks.
        {
            System.setProperty("java.net.preferIPv4Stack", "true");
        }
        String vendor = System.getProperty("java.vendor", "missing vendor");
        String javaVersion = System.getProperty("java.version", "missing java version");
        String jvmVersion = System.getProperty("java.vm.version", "missing jvm version");
        monitor.message(String.format("JVM info: %s - %s - %s", vendor, javaVersion, jvmVersion));
        monitor.message("java.net.preferIPv4Stack=" + System.getProperty("java.net.preferIPv4Stack"));
        monitor.message("Current Time: " + new SimpleDateFormat("dd/MM/yyyy HH:mm:ss").format(new Date()));

        File installer = new File(SimpleInstaller.class.getProtectionDomain().getCodeSource().getLocation().toURI());
        if (installer.getAbsolutePath().contains("!/")) {
            monitor.stage("Due to java limitation, please do not run this jar in a folder ending with !");
            monitor.message(installer.getAbsolutePath());
            return;
        }

        OptionParser parser = new OptionParser();
        OptionSpec<File> clientInstallOption = parser.acceptsAll(Arrays.asList("installClient", "install-client"), "Install a client to the specified directory, defaulting to the MC installation directory").withOptionalArg().ofType(File.class).defaultsTo(getMCDir());
        OptionSpec<File> serverInstallOption = parser.acceptsAll(Arrays.asList("installServer", "install-server"), "Install a server to the current directory").withOptionalArg().ofType(File.class).defaultsTo(new File("."));

        OptionSpec<Void> serverStarterOption = parser.acceptsAll(Arrays.asList("server-starter", "server.jar", "server-jar"), "Download the server starter jar for arg-free executable launches");

        OptionSpec<File> fatInstallerOption = parser.acceptsAll(Arrays.asList("fat-installer", "fat", "generate-fat"), "Generate a fat installer jar").withOptionalArg().ofType(File.class).defaultsTo(new File(installer.getParent(), installer.getName().replace(".jar", "-fat.jar")));
        OptionSpec<Void> fatIncludeMC = parser.acceptsAll(Arrays.asList("fat-include-minecraft"), "Include the Minecraft client / server jar in the fat installer").availableIf(fatInstallerOption);
        OptionSpec<Void> fatIncludeMCLibs = parser.acceptsAll(Arrays.asList("fat-include-minecraft-libs"), "Include the Minecraft libraries in the fat installer").availableIf(fatInstallerOption);
        OptionSpec<Void> fatIncludeInstallerLibs = parser.acceptsAll(Arrays.asList("fat-include-installer-libs"), "Include the installer libraries in the fat installer").availableIf(fatInstallerOption);
        OptionSpec<Void> fatOffline = parser.acceptsAll(Arrays.asList("fat-offline", "gen-offline", "generate-offline", "gf"), "Generate an online fat installer");

        OptionSpec<Void> helpOption = parser.acceptsAll(Arrays.asList("h", "help"), "Help with this installer");
        OptionSpec<Void> offlineOption = parser.accepts("offline", "Don't attempt any network calls");
        OptionSpec<Void> debugOption = parser.accepts("debug", "Run in debug mode -- don't delete any files");
        OptionSpec<Void> skipHashCheckOption = parser.accepts("skip-hash-check", "Skips the hash check when verifying outputs");
        OptionSpec<URL> mirrorOption = parser.accepts("mirror", "Use a specific mirror URL").withRequiredArg().ofType(URL.class);
        OptionSet optionSet = parser.parse(args);

        if (optionSet.has(helpOption)) {
            parser.printHelpOn(System.out);
            return;
        }

        debug = optionSet.has(debugOption);
        if (optionSet.has(mirrorOption)) {
            mirror = optionSet.valueOf(mirrorOption);
        }
        skipHashCheck = optionSet.has(skipHashCheckOption);

        boolean isOffline = optionSet.has(offlineOption);
        if (Files.isRegularFile(installer.toPath())) {
            try (JarFile jf = new JarFile(installer)) {
                isOffline = isOffline | Boolean.parseBoolean(jf.getManifest().getMainAttributes().getValue("Offline"));
            }
        }
        if (isOffline) {
            DownloadUtils.OFFLINE_MODE = true;
            monitor.message("ENABLING OFFLINE MODE");
        } else {
            for (String host : new String[] {
                    "maven.neoforged.net",
                    "libraries.minecraft.net",
                    "launchermeta.mojang.com",
                    "piston-meta.mojang.com",
                    "sessionserver.mojang.com",
            }) {
                monitor.message("Host: " + host + " [" + DownloadUtils.getIps(host).stream().collect(Collectors.joining(", ")) + "]");
            }
            FixSSL.fixup(monitor);
        }

        Actions action = null;
        File target = null;
        if (optionSet.has(serverInstallOption)) {
            action = Actions.SERVER;
            target = optionSet.valueOf(serverInstallOption);
            ServerInstall.serverStarterJar = optionSet.has(serverStarterOption);
        } else if (optionSet.has(clientInstallOption)) {
            action = Actions.CLIENT;
            target = optionSet.valueOf(clientInstallOption);
        } else if (optionSet.has(fatInstallerOption) || optionSet.has(fatOffline)) {
            action = Actions.FAT_INSTALLER;
            target = optionSet.valueOf(fatInstallerOption);

            if (optionSet.has(fatIncludeMC) || optionSet.has(fatOffline)) {
                FatInstallerAction.OPTIONS.add(FatInstallerAction.Options.MC_JAR);
            }
            if (optionSet.has(fatIncludeMCLibs) || optionSet.has(fatOffline)) {
                FatInstallerAction.OPTIONS.add(FatInstallerAction.Options.MC_LIBS);
            }
            if (optionSet.has(fatIncludeInstallerLibs) || optionSet.has(fatOffline)) {
                FatInstallerAction.OPTIONS.add(FatInstallerAction.Options.INSTALLER_LIBS);
            }
        }

        if (action == null && GraphicsEnvironment.isHeadless()) {
            monitor.message("No action was specified but headless environment was detected... Installing server");
            action = Actions.SERVER;
            target = optionSet.valueOf(serverInstallOption);
            ServerInstall.serverStarterJar = true;
        }

        if (action != null) {
            try {
                SimpleInstaller.headless = true;
                monitor.message("Target Directory: " + target);
                InstallV1 install = Util.loadInstallProfile();
                if (!action.getAction(install, monitor).run(target, a -> true, installer)) {
                    monitor.stage("There was an error during installation");
                    System.exit(1);
                } else {
                    monitor.message(action.getSuccess());
                    monitor.stage("You can delete this installer file now if you wish");
                }
            } catch (Throwable e) {
                monitor.stage("A problem installing was detected, install cannot continue");
                System.exit(1);
            }
        } else
            launchGui(monitor, installer);
    }

    public static File getMCDir() {
        if (System.getenv("mcdir") != null)
            return new File(System.getenv("mcdir"));
        String userHomeDir = System.getProperty("user.home", ".");
        String osType = System.getProperty("os.name").toLowerCase(Locale.ENGLISH);
        String mcDir = ".minecraft";
        if (osType.contains("win") && System.getenv("APPDATA") != null)
            return new File(System.getenv("APPDATA"), mcDir);
        else if (osType.contains("mac"))
            return new File(new File(new File(userHomeDir, "Library"), "Application Support"), "minecraft");

        File candidate = new File(userHomeDir, mcDir);
        if (!candidate.exists() && osType.contains("linux")) {
            File flatPakCandidate = new File(userHomeDir, ".var/app/com.mojang.Minecraft/.minecraft");
            if (flatPakCandidate.exists())
            {
                System.out.println("Flatpak path detected. Using it as the primary Minecraft directory.");
                return flatPakCandidate;
            }
        }

        return candidate;
    }

    private static void launchGui(ProgressCallback monitor, File installer) {
        try {
            UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName());
        } catch (Exception e) {}

        try {
            InstallV1 profile = Util.loadInstallProfile();
            InstallerPanel panel = new InstallerPanel(getMCDir(), profile, installer);
            panel.run(monitor);
        } catch (Throwable e) {
            JOptionPane.showMessageDialog(null, "Something went wrong while installing.<br />Check log for more details:<br/>" + e.toString(), "Error", JOptionPane.ERROR_MESSAGE);
        }
    }

    private static OutputStream getLog() throws FileNotFoundException {
        File f = new File(SimpleInstaller.class.getProtectionDomain().getCodeSource().getLocation().getFile());
        File output;
        if (f.isFile()) output = new File(f.getName() + ".log");
        else output = new File("installer.log");
        System.out.println("Outputting log to file " + output);

        return new BufferedOutputStream(new FileOutputStream(output));
    }

    public static void hookStdOut(ProgressCallback monitor) {
        final Pattern endingWhitespace = Pattern.compile("\\r?\\n$");
        final OutputStream monitorStream = new OutputStream() {
            private StringBuffer buffer = new StringBuffer();

            @Override
            public void write(byte[] buf, int off, int len) {
                for (int i = off; i < off + len; i++) {
                    write(buf[i]);
                }
            }

            @Override
            public void write(byte[] b) {
                write(b, 0, b.length);
            }

            @Override
            public void write(int b) {
                if (b == '\r') return; // Ignore CR
                if (b == '\n') {
                    final String message = endingWhitespace.matcher(buffer.toString()).replaceAll("");
                    if (!message.isEmpty()) {
                        monitor.message(message);
                    }

                    buffer = new StringBuffer();
                } else {
                    buffer.append((char) b);
                }
            }
        };

        System.setOut(new PrintStream(monitorStream));
        System.setErr(new PrintStream(new ProgressInterceptor(monitorStream, new ProgressManager() {
            @Override
            public void setMaxProgress(int maxProgress) {
                monitor.getStepProgress().setMaxProgress(maxProgress);
            }

            @Override
            public void setProgress(int progress) {
                monitor.getStepProgress().progress(progress);
            }

            @Override
            public void setPercentageProgress(double percentage) {
                monitor.getStepProgress().percentageProgress(percentage);
            }

            @Override
            public void setStep(String step) {
                monitor.message(monitor.getCurrentStep() + ": " + step, ProgressCallback.MessagePriority.HIGH);
            }

            @Override
            public void setIndeterminate(boolean indeterminate) {
                monitor.getStepProgress().setIndeterminate(false);
            }
        })));

        System.setProperty(ProgressReporter.ENABLED_PROPERTY, "true");
    }
}

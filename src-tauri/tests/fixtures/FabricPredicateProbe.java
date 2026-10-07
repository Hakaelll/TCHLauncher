import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Base64;
import net.fabricmc.loader.api.Version;
import net.fabricmc.loader.api.VersionParsingException;
import net.fabricmc.loader.api.metadata.version.VersionPredicate;

/** Independent Fabric API oracle. Does not instantiate a loader or run Minecraft. */
public final class FabricPredicateProbe {
    public static void main(String[] args) throws Exception {
        for (String line : Files.readAllLines(Paths.get(args[0]), StandardCharsets.UTF_8)) {
            String[] fields = line.split("\t", -1);
            if (fields.length != 2) throw new IllegalArgumentException("Invalid fixture");
            String version = new String(Base64.getDecoder().decode(fields[0]), StandardCharsets.UTF_8);
            String range = new String(Base64.getDecoder().decode(fields[1]), StandardCharsets.UTF_8);
            try {
                System.out.println(VersionPredicate.parse(range).test(Version.parse(version)) ? "true" : "false");
            } catch (VersionParsingException invalid) {
                System.out.println("unknown");
            }
        }
    }
}

import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Base64;
import java.util.Properties;

/** Independent oracle: no Minecraft code, network, EULA or world generation. */
public final class SeedPropertiesProbe {
    public static void main(String[] args) throws Exception {
        Properties properties = new Properties();
        try (InputStream input = Files.newInputStream(Paths.get(args[0]))) {
            properties.load(input);
        }
        String seed = properties.getProperty("level-seed");
        if (seed == null) throw new IllegalStateException("Missing seed");
        System.out.println(Base64.getEncoder().encodeToString(seed.getBytes(StandardCharsets.UTF_8)));
        if (!"true".equals(properties.getProperty("online-mode"))) {
            throw new IllegalStateException("Authentication value changed");
        }
    }
}

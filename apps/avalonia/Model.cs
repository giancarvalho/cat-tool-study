using System.Text.Json;
using System.Text.Json.Serialization;

namespace CatStudy;

/// <summary>Nó inline do núcleo: texto (k="t") ou tag (k="g"). Mesmo formato JSON dos outros candidatos.</summary>
public class Inline
{
    [JsonPropertyName("k")] public string K { get; set; } = "t";
    [JsonPropertyName("v")] public string? V { get; set; }
    [JsonPropertyName("b")] public bool? B { get; set; }
    [JsonPropertyName("i")] public bool? I { get; set; }
    [JsonPropertyName("id")] public int? Id { get; set; }
    [JsonPropertyName("tipo")] public string? Tipo { get; set; }
    [JsonPropertyName("rotulo")] public string? Rotulo { get; set; }

    [JsonIgnore] public bool IsTag => K == "g";
    public static Inline Texto(string s) => new() { K = "t", V = s };
    public string ChipLabel => Tipo == "abre" ? Id.ToString()! : Tipo == "fecha" ? "/" + Id : Rotulo ?? "";
    public Inline Clone() => new() { K = K, V = V, B = B, I = I, Id = Id, Tipo = Tipo, Rotulo = Rotulo };
}

public class Seg
{
    [JsonPropertyName("id")] public long Id { get; set; }
    [JsonPropertyName("bloco_id")] public long BlocoId { get; set; }
    [JsonPropertyName("ordem")] public double Ordem { get; set; }
    [JsonPropertyName("origem")] public List<Inline> Origem { get; set; } = new();
    [JsonPropertyName("destino")] public List<Inline> Destino { get; set; } = new();
    [JsonPropertyName("estado")] public string Estado { get; set; } = "";
    [JsonPropertyName("versao")] public long Versao { get; set; }
    [JsonIgnore] public List<Inline> Base { get; set; } = new();
    [JsonIgnore] public int Indice { get; set; }
}

public class Bloco
{
    [JsonPropertyName("id")] public long Id { get; set; }
    [JsonPropertyName("tipo")] public string Tipo { get; set; } = "";
    [JsonPropertyName("nivel")] public int Nivel { get; set; }
}

public class ResultadoBusca
{
    [JsonPropertyName("origem")] public string Origem { get; set; } = "";
    [JsonPropertyName("destino")] public string Destino { get; set; } = "";
    [JsonPropertyName("pontuacao")] public double Pontuacao { get; set; }
}

public static class Json
{
    public static readonly JsonSerializerOptions Opt = new() { DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull, PropertyNameCaseInsensitive = true };
    public static List<Inline> CloneList(List<Inline> l) => l.Select(x => x.Clone()).ToList();
    public static bool Igual(List<Inline> a, List<Inline> b) => JsonSerializer.Serialize(a, Opt) == JsonSerializer.Serialize(b, Opt);
}

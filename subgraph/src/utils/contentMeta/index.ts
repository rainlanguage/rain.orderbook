import { BigInt, Bytes, JSONValue, JSONValueKind, TypedMap } from "@graphprotocol/graph-ts";
import { ContentMetaV1 } from "../../../generated/schema";
import { getKeccak256FromBytes, isHexString } from "@rainprotocol/subgraph-utils";
import { CBOREncoder } from "@rainprotocol/assemblyscript-cbor";

export class ContentMeta {
  rainMetaId: Bytes;
  encodedData: Bytes = Bytes.empty();
  payload: Bytes = Bytes.empty();
  // eslint-disable-next-line @typescript-eslint/ban-types
  magicNumber: BigInt = BigInt.zero();
  contentType: string = "";
  contentEncoding: string = "";
  contentLanguage: string = "";

  private contentTypeAdded: boolean = false;
  private contentEncodingAdded: boolean = false;
  private contentLanguageAdded: boolean = false;

  private metaContent: ContentMetaV1 = new ContentMetaV1(Bytes.empty());
  private metaStored: boolean = false;

  constructor(
    metaContentV1Object: TypedMap<string, JSONValue>,
    rainMetaID: Bytes
  ) {
    const payload = metaContentV1Object.get("0");
    const magicNumber = metaContentV1Object.get("1");
    const contentType = metaContentV1Object.get("2");
    const contentEncoding = metaContentV1Object.get("3");
    const contentLanguage = metaContentV1Object.get("4");

    // RainMetaV1 ID
    this.rainMetaId = rainMetaID;

    // Mandatories keys
    if (payload) {
      let auxPayload = payload.toString();
      if (auxPayload.startsWith("h'")) {
        auxPayload = auxPayload.replace("h'", "");
      }
      if (auxPayload.endsWith("'")) {
        auxPayload = auxPayload.replace("'", "");
      }

      this.payload = Bytes.fromHexString(auxPayload);
    }

    // if (payload) this.payload = payload.toString();
    if (magicNumber) this.magicNumber = magicNumber.toBigInt();

    // Keys optionals
    if (contentType) {
      this.contentTypeAdded = true;
      this.contentType = contentType.toString();
    }

    if (contentEncoding) {
      this.contentEncodingAdded = true;
      this.contentEncoding = contentEncoding.toString();
    }

    if (contentLanguage) {
      this.contentLanguageAdded = true;
      this.contentLanguage = contentLanguage.toString();
    }
  }

  /**
   * Validate that the keys exist on the map
   */
  static validate(metaContentV1Object: TypedMap<string, JSONValue>): boolean {
    const payload = metaContentV1Object.get("0");
    const magicNumber = metaContentV1Object.get("1");
    const contentType = metaContentV1Object.get("2");
    const contentEncoding = metaContentV1Object.get("3");
    const contentLanguage = metaContentV1Object.get("4");

    // Only payload and magicNumber are mandatory on RainMetaV1
    // See: https://github.com/rainprotocol/specs/blob/main/metadata-v1.md
    if (payload && magicNumber) {
      if (
        payload.kind == JSONValueKind.STRING ||
        magicNumber.kind == JSONValueKind.NUMBER
      ) {
        // Check if payload is a valid Bytes (hexa)
        let auxPayload = payload.toString();
        if (auxPayload.startsWith("h'")) {
          auxPayload = auxPayload.replace("h'", "");
        }
        if (auxPayload.endsWith("'")) {
          auxPayload = auxPayload.replace("'", "");
        }

        // If the payload is not a valid bytes value
        if (!isHexString(auxPayload)) {
          return false;
        }

        // Check the type of optionals keys
        if (contentType) {
          if (contentType.kind != JSONValueKind.STRING) {
            return false;
          }
        }
        if (contentEncoding) {
          if (contentEncoding.kind != JSONValueKind.STRING) {
            return false;
          }
        }
        if (contentLanguage) {
          if (contentLanguage.kind != JSONValueKind.STRING) {
            return false;
          }
        }

        return true;
      }
    }

    return false;
  }

  private getContentId(): Bytes {
    // Values as Bytes
    const encoder = new CBOREncoder();
    // Initially, the map always have two keys/values (payload and magic number)
    let mapLength = 2;

    if (this.contentTypeAdded) mapLength += 1;
    if (this.contentEncodingAdded) mapLength += 1;
    if (this.contentLanguageAdded) mapLength += 1;

    encoder.addObject(mapLength);

    // -- Add key 0 (payload)
    encoder.addUint8(0);
    encoder.addBytes(this.payload);

    // -- Add key 1 (magic number)
    encoder.addUint8(1);
    encoder.addUint64(this.magicNumber.toU64());

    if (this.contentTypeAdded) {
      // -- Add key 2 (Content-Type)
      encoder.addUint8(2);
      encoder.addString(this.contentType);
    }

    if (this.contentEncodingAdded) {
      // -- Add key 3 (Content-Encoding)
      encoder.addUint8(3);
      encoder.addString(this.contentEncoding);
    }

    if (this.contentLanguageAdded) {
      // -- Add key 4 (Content-Language)
      encoder.addUint8(4);
      encoder.addString(this.contentLanguage);
    }

    this.encodedData = Bytes.fromHexString(encoder.serializeString());

    return getKeccak256FromBytes(this.encodedData);
  }

  /**
   * Create or generate a ContentMetaV1 entity based on the current fields:
   *
   * - If the ContentMetaV1 does not exist, create the ContentMetaV1 entity and
   * made the relation to the rainMetaId.
   *
   * - If the ContentMetaV1 does exist, add the relation to the rainMetaId.
   */
  generate(): ContentMetaV1 {
    const contentId = this.getContentId();

    let metaContent = ContentMetaV1.load(contentId);

    if (!metaContent) {
      metaContent = new ContentMetaV1(contentId);

      metaContent.rawBytes = this.encodedData;
      metaContent.magicNumber = this.magicNumber;
      metaContent.payload = this.payload;
      metaContent.parents = [];

      if (this.contentType != "") metaContent.contentType = this.contentType;

      if (this.contentEncoding != "")
        metaContent.contentEncoding = this.contentEncoding;

      if (this.contentLanguage != "")
        metaContent.contentLanguage = this.contentLanguage;
    }

    const auxParents = metaContent.parents;
    if (!auxParents.includes(this.rainMetaId)) auxParents.push(this.rainMetaId);
    metaContent.parents = auxParents;

    this.metaContent = metaContent;
    this.metaStored = true;

    return this.metaContent;
  }

  saveMeta(): void {
    if (this.metaStored && this.metaContent.id.notEqual(Bytes.empty())) {
      this.metaContent.save();
    }
  }
}